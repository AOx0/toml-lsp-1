# aoxo-toml

Toy parser and LSP for a subset of TOML (excludes datetimes).

```py
Toml = Expr*
Expr =
      TableArray
    | Table
    | KeyVal

TableArray = '[[' Key ']]' '\n' (KeyVal '\n')*
Table = '[' Key ']' '\n' (KeyVal '\n')*

KeyVal = Key '=' Value

Key = KeyPart ('.' KeyPart)*
KeyPart = 'str_key' | 'key'

Value =
      'string'
    | 'number'
    | 'bool'
    | Array
    | TableInline

Array = '[' (Value ( ',' | '\n' ))* ']'
TableInline = '{' KeyVal? (',' KeyVal)* '}'
```
