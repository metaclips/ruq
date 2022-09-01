RUQ (Rust Universal Querier), is a lightweight JSON, TOML, and YAML processor. ruq uses jq like syntax but works with json, and yaml.

# User Guide
Get a value
echo "{"foo": 0}" | ruq --filter '.foo' --from 'json'

Get an indexed value
echo '[{"foo": 0}, {"foo": 1}]' | ruq --filter '.[1].foo'

Conversion
echo '[{"foo": 0}, {"foo": 1}]' | ruq --filter '.[1]' --from json --to toml

Arthmetic
echo '{}' | ruq --filter '{"a": 1} + {"b": 2} + {"c": 3} + {"a": 42}'

Length
echo '[{"foo": 0}, {"foo": 1}]' | ruq --filter '.|length'
