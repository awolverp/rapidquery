#!/usr/bin/env bash
rs="src/dialect/base.rs"
pyi="rapidquery/dialect.pyi"
missing=0

miss() {
    echo "MISSING in stub: $1"
    missing=1
}

# 1. Python classes: pyclass("rapidquery.dialect.Name")
grep -oE 'pyclass\("rapidquery\.dialect\.[A-Za-z]+"' "$rs" | sed 's/.*\.//; s/"//' |
while read -r c; do grep -q "^class $c\b" "$pyi" || miss "class $c"; done

# 2. Constructor fields: "name:" lines inside pyo3_constructor! { ... }
awk '/pyo3_constructor!/{on=1} on&&/^}/{on=0} on' "$rs" |
grep -oE '^\s+[a-z_]+:' | tr -d ' :' | sort -u |
while read -r f; do grep -q "def $f(" "$pyi" || miss "field $f"; done

# 3. Flags: "field: bits|enum Type { A, B }" -> OPT_<FIELD>_<VARIANT>
tr '\n' ' ' < "$rs" |
grep -oE '[a-z_]+: (bits|enum) [A-Za-z]+ \{[^}]*\}' |
while read -r line; do
  field=$(echo "${line%%:*}" | tr a-z A-Z)
  variants=$(echo "$line" | sed 's/.*{//; s/}//; s/,/ /g')
  for v in $variants; do
    snake=$(echo "$v" | sed 's/\([a-z0-9]\)\([A-Z]\)/\1_\2/g' | tr a-z A-Z)
    grep -q "^OPT_${field}_${snake}:" "$pyi" || miss "OPT_${field}_${snake}"
  done
done

exit $missing
