use saphyr::{LoadableYamlNode, Yaml, YamlEmitter};

#[allow(clippy::similar_names)]
#[test]
fn test_emit_simple() {
    let s = "
# comment
a0 bb: val
a1:
    b1: 4
    b2: d
a2: 4 # i'm comment
a3: [1, 2, 3]
a4:
    - [a1, a2]
    - 2
";

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    println!("original:\n{s}");
    println!("emitted:\n{writer}");
    let docs_new = match Yaml::load_from_str(&writer) {
        Ok(y) => y,
        Err(e) => panic!("{}", e),
    };
    let doc_new = &docs_new[0];

    assert_eq!(doc, doc_new);
}

#[test]
fn test_emit_complex() {
    let s = r"
catalogue:
  product: &coffee   { name: Coffee,    price: 2.5  ,  unit: 1l  }
  product: &cookies  { name: Cookies!,  price: 3.40 ,  unit: 400g}

products:
  *coffee :
    amount: 4
  *cookies :
    amount: 4
  [1,2,3,4]:
    array key
  2.4:
    real key
  true:
    bool key
  {}:
    empty hash key
            ";
    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    let docs_new = match Yaml::load_from_str(&writer) {
        Ok(y) => y,
        Err(e) => panic!("{}", e),
    };
    let new_doc = &docs_new[0];
    assert_eq!(doc, new_doc);
}

#[test]
fn test_emit_avoid_quotes() {
    let s = r#"---
a7: 你好
boolean: "true"
boolean2: "false"
date: 2014-12-31
empty_string: ""
empty_string1: " "
empty_string2: "    a"
empty_string3: "    a "
exp: "12e7"
field: ":"
field2: "{"
field3: "\\"
field4: "\n"
field5: "can't avoid quote"
float: "2.6"
int: "4"
nullable: "null"
nullable2: "~"
products:
  "*coffee":
    amount: 4
  "*cookies":
    amount: 4
  ".milk":
    amount: 1
  "2.4": real key
  "[1,2,3,4]": array key
  "true": bool key
  "{}": empty hash key
x: test
y: avoid quoting here
z: string with spaces"#;

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }

    assert_eq!(s, writer, "actual:\n\n{writer}\n");
}

#[test]
fn emit_quoted_bools() {
    let input = r#"---
string0: yes
string1: no
string2: "true"
string3: "false"
string4: "~"
null0: ~
[true, True, TRUE, false, False, FALSE]: real_bools
[y,Y,yes,Yes,YES,n,N,no,No,NO,on,On,ON,off,Off,OFF]: false_bools
bool0: true
bool1: false"#;
    let expected = r#"---
string0: "yes"
string1: "no"
string2: "true"
string3: "false"
string4: "~"
null0: ~
? - true
  - true
  - true
  - false
  - false
  - false
: real_bools
? - y
  - Y
  - "yes"
  - "Yes"
  - "YES"
  - n
  - N
  - "no"
  - "No"
  - "NO"
  - "on"
  - "On"
  - "ON"
  - "off"
  - "Off"
  - "OFF"
: false_bools
bool0: true
bool1: false"#;

    let docs = Yaml::load_from_str(input).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }

    assert_eq!(
        expected, writer,
        "expected:\n{expected}\nactual:\n{writer}\n",
    );
}

/// Ensure that whole floats such as 100.0 emit as a float (not int).
#[test]
fn test_emit_whole_floats() {
    let input = r#"---
value: 100.0"#;
    let expected = r#"---
value: 100.0"#;
    let docs = Yaml::load_from_str(input).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    assert_eq!(
        expected, writer,
        "expected:\n{expected}\nactual:\n{writer}\n",
    );
}

#[test]
fn test_sort_keys() {
    let input = r"---
z: last
a: first
item10: ten
item2: two
nested:
  z: 1
  a: 2
sequence:
  - z: 1
    a: 2";
    let expected = r"---
a: first
item2: two
item10: ten
nested:
  a: 2
  z: 1
sequence:
  - a: 2
    z: 1
z: last";
    let docs = Yaml::load_from_str(input).unwrap();
    let mut writer = String::new();
    let mut emitter = YamlEmitter::new(&mut writer);

    assert!(!emitter.is_sorting_keys());
    emitter.sort_keys(true);
    assert!(emitter.is_sorting_keys());
    emitter.dump(&docs[0]).unwrap();

    assert_eq!(expected, writer);
    let original_keys: Vec<_> = docs[0]
        .as_mapping()
        .unwrap()
        .keys()
        .map(|key| key.as_str().unwrap())
        .collect();
    assert_eq!(
        original_keys,
        ["z", "a", "item10", "item2", "nested", "sequence"]
    );
}

#[test]
fn test_sort_keys_orders_scalars_before_yaml_only_keys() {
    let input = r"---
~: null
string: string
? [z, a]
: sequence
true: boolean true
1: integer one
1.0: float one
0.5: half
false: boolean false
-1: negative
? { z: 1, a: 2 }
: mapping";
    let docs = Yaml::load_from_str(input).unwrap();
    let mut writer = String::new();
    let mut emitter = YamlEmitter::new(&mut writer);
    emitter.sort_keys(true);
    emitter.dump(&docs[0]).unwrap();

    let ordered_fragments = [
        "-1: negative",
        "false: boolean false",
        "0.5: half",
        "true: boolean true",
        "1: integer one",
        "1.0: float one",
        "string: string",
        "~: ~",
        "? - z",
        "? a: 2",
    ];
    let positions: Vec<_> = ordered_fragments
        .iter()
        .map(|fragment| {
            writer
                .find(fragment)
                .unwrap_or_else(|| panic!("missing {fragment:?} in:\n{writer}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{writer}"
    );
}

#[test]
fn test_empty_and_nested() {
    test_empty_and_nested_flag(false);
}

#[test]
fn test_empty_and_nested_compact() {
    test_empty_and_nested_flag(true);
}

fn test_empty_and_nested_flag(compact: bool) {
    let s = if compact {
        r"---
a:
  b:
    c: hello
  d: {}
e:
  - f
  - g
  - h: []"
    } else {
        r"---
a:
  b:
    c: hello
  d: {}
e:
  - f
  - g
  -
    h: []"
    };

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.compact(compact);
        emitter.dump(doc).unwrap();
    }

    assert_eq!(s, writer);
}

#[test]
fn test_nested_sequences() {
    let s = r"---
a:
  - b
  - - c
    - d
    - - e
      - f";

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    println!("original:\n{s}");
    println!("emitted:\n{writer}");

    assert_eq!(s, writer);
}

#[test]
fn test_deeply_nested_sequences() {
    let s = r"---
a:
  - b
  - - c
    - d
    - - e
      - - f
      - - e";

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    println!("original:\n{s}");
    println!("emitted:\n{writer}");

    assert_eq!(s, writer);
}

#[test]
fn test_nested_mappings() {
    let s = r"---
a:
  b:
    c:
      d:
        e: f";

    let docs = Yaml::load_from_str(s).unwrap();
    let doc = &docs[0];
    let mut writer = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut writer);
        emitter.dump(doc).unwrap();
    }
    println!("original:\n{s}");
    println!("emitted:\n{writer}");

    assert_eq!(s, writer);
}
