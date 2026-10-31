"""schema 与示例的深度校验（被 scripts/validate-contracts.ps1 调用）。

优先用 jsonschema 库（CI 会 pip install）；本机没有时退回内置的**最小校验器**，
它只实现本仓库 schema 用到的关键字，够用来防“示例与 schema 漂移”。
退出码：0 = 通过；1 = 有不一致。
"""
import glob
import json
import os
import re
import sys

SCHEMA_SUFFIX = ".schema.json"


def load_json(path):
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def resolve_ref(ref, root):
    if not ref.startswith("#/"):
        raise ValueError("only local refs are supported: " + ref)
    node = root
    for part in ref[2:].split("/"):
        node = node[part]
    return node


def type_ok(value, name):
    if name == "object":
        return isinstance(value, dict)
    if name == "array":
        return isinstance(value, list)
    if name == "string":
        return isinstance(value, str)
    if name == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if name == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if name == "boolean":
        return isinstance(value, bool)
    if name == "null":
        return value is None
    return True


def validate(instance, schema, root, path, errors):
    if schema is True or schema is None:
        return
    if not isinstance(schema, dict):
        errors.append((path, "schema node is not an object"))
        return

    if "$ref" in schema:
        validate(instance, resolve_ref(schema["$ref"], root), root, path, errors)

    if "const" in schema and instance != schema["const"]:
        errors.append((path, "expected const %r, got %r" % (schema["const"], instance)))

    if "enum" in schema and instance not in schema["enum"]:
        errors.append((path, "value %r not in enum" % (instance,)))

    if "type" in schema:
        types = schema["type"]
        if isinstance(types, str):
            types = [types]
        if not any(type_ok(instance, t) for t in types):
            errors.append((path, "expected type %r, got %r" % (types, type(instance).__name__)))
            return

    if isinstance(instance, str):
        if "minLength" in schema and len(instance) < schema["minLength"]:
            errors.append((path, "string shorter than minLength %d" % schema["minLength"]))
        if "pattern" in schema and re.search(schema["pattern"], instance) is None:
            errors.append((path, "string does not match pattern %r" % schema["pattern"]))

    if isinstance(instance, (int, float)) and not isinstance(instance, bool):
        if "minimum" in schema and instance < schema["minimum"]:
            errors.append((path, "number below minimum %r" % schema["minimum"]))
        if "maximum" in schema and instance > schema["maximum"]:
            errors.append((path, "number above maximum %r" % schema["maximum"]))

    if isinstance(instance, list):
        if "minItems" in schema and len(instance) < schema["minItems"]:
            errors.append((path, "array shorter than minItems %d" % schema["minItems"]))
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            errors.append((path, "array longer than maxItems %d" % schema["maxItems"]))
        if "items" in schema:
            for index, item in enumerate(instance):
                validate(item, schema["items"], root, path + "[%d]" % index, errors)

    if isinstance(instance, dict):
        if "required" in schema:
            for key in schema["required"]:
                if key not in instance:
                    errors.append((path, "missing required property %r" % key))
        properties = schema.get("properties", {})
        for key, value in instance.items():
            if key in properties:
                validate(value, properties[key], root, path + "." + key, errors)
            elif schema.get("additionalProperties") is False:
                errors.append((path, "unexpected property %r" % key))

    if "allOf" in schema:
        for sub in schema["allOf"]:
            validate(instance, sub, root, path, errors)

    if "anyOf" in schema:
        branches = []
        for sub in schema["anyOf"]:
            local = []
            validate(instance, sub, root, path, local)
            branches.append(local)
        if all(branches):
            errors.append((path, "no anyOf branch matched"))

    if "oneOf" in schema:
        passed = 0
        for sub in schema["oneOf"]:
            local = []
            validate(instance, sub, root, path, local)
            if not local:
                passed += 1
        if passed != 1:
            errors.append((path, "oneOf matched %d branches (expected exactly 1)" % passed))


def check_schema(schema, root, path, errors):
    """最小结构检查：$ref 可解析、oneOf 非空。"""
    if isinstance(schema, dict):
        if "$ref" in schema:
            try:
                resolve_ref(schema["$ref"], root)
            except Exception as exc:  # noqa: BLE001
                errors.append((path, "unresolvable $ref: %s" % exc))
        for key, value in schema.items():
            if key in ("properties", "$defs"):
                for name, sub in value.items():
                    check_schema(sub, root, path + "/" + name, errors)
            elif key in ("items",):
                check_schema(value, root, path + "/items", errors)
            elif key in ("oneOf", "anyOf", "allOf") and isinstance(value, list):
                if not value:
                    errors.append((path, "%s is empty" % key))
                for index, sub in enumerate(value):
                    check_schema(sub, root, path + "/%s[%d]" % (key, index), errors)


def main():
    root_dir = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    schema_dir = os.path.join(root_dir, "contracts", "schemas")
    example_dir = os.path.join(schema_dir, "examples")

    schemas = {}
    for path in sorted(glob.glob(os.path.join(schema_dir, "*" + SCHEMA_SUFFIX))):
        stem = os.path.basename(path)[: -len(SCHEMA_SUFFIX)]
        schemas[stem] = (path, load_json(path))

    failures = 0
    for stem, (path, schema) in schemas.items():
        errors = []
        check_schema(schema, schema, "", errors)
        if errors:
            failures += 1
            for where, message in errors[:5]:
                print("[FAIL] %s%s: %s" % (os.path.basename(path), where, message))

    try:
        import jsonschema  # noqa: F401
        from jsonschema.validators import Draft202012Validator

        engine = "jsonschema"
    except ImportError:
        engine = "builtin"

    print("[info] validator engine: %s" % engine)

    examples = sorted(glob.glob(os.path.join(example_dir, "*.json")))
    for example in examples:
        name = os.path.basename(example)
        stem = name.split(".")[0]
        if stem not in schemas:
            continue  # 命名问题由 validate-contracts.ps1 报
        _, schema = schemas[stem]
        instance = load_json(example)
        if engine == "jsonschema":
            validator = Draft202012Validator(schema)
            found = sorted(
                validator.iter_errors(instance), key=lambda e: list(e.absolute_path)
            )
            messages = [
                "%s at /%s" % (e.message, "/".join(str(p) for p in e.absolute_path))
                for e in found
            ]
        else:
            errors = []
            validate(instance, schema, schema, "/" + stem, errors)
            messages = ["%s at %s" % (message, where) for where, message in errors]
        if messages:
            failures += 1
            for message in messages[:5]:
                print("[FAIL] %s: %s" % (name, message))
        else:
            print("[ok]   %s 符合 %s.schema.json" % (name, stem))

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
