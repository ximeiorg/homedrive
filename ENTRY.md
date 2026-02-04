```
sea-orm-cli generate entity \
    --database-url 'sqlite:homedrive.db' \
    --output-dir ./crates/store/src/entity \
    --entity-format dense \
    --big-integer-type i64
```