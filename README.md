Mana Base
---------

## Usage

Requires Rust compiler and Cargo tools. To generate pages under default `target/www` path:

```
./generate.sh
```

Invalid card names and other tagging issues will be displayed as warnings.

## Tagging Cards

Full card list with associated tags is located at [config/card-tags.toml](config/card-tags.toml).
Tag configuration under [config/tags.toml](config/tags.toml).

Scyrfall search string:

```
oracle:land or oracle:add or oracle:mana or type:land
```

## Page Templates

Compile-time page templates are under the `templates` folder and use a Jinja-like syntax.