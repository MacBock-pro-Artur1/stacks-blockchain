use vm::ast::parse;
use vm::representations::SymbolicExpression;
use vm::analysis::type_checker::{TypeResult, TypeChecker, TypingContext};
use vm::analysis::{AnalysisDatabase};
use vm::analysis::errors::CheckErrors;
use vm::analysis::mem_type_check;
use vm::analysis::type_check;
use vm::analysis::types::ContractAnalysis;
use vm::contexts::{OwnedEnvironment};
use vm::types::{Value, PrincipalData, TypeSignature, AtomTypeIdentifier, FunctionType, QualifiedContractIdentifier};

mod assets;
mod contracts;

fn type_check_helper(exp: &str) -> TypeResult {
    let mut db = AnalysisDatabase::memory();
    let contract_id = QualifiedContractIdentifier::transient();
    let exp = parse(&contract_id, exp).unwrap();
    db.execute(|db| {
        let mut type_checker = TypeChecker::new(db);
        
        let contract_context = TypingContext::new();
        type_checker.type_check(&exp[0], &contract_context)
    })
}


#[test]
fn test_get_block_info(){
    let good = ["(get-block-info time 1)",
                "(get-block-info time (* 2 3))"];
    let bad = ["(get-block-info none 1)",
               "(get-block-info time 'true)",
               "(get-block-info time)"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_simple_arithmetic_checks() {
    let good = ["(>= (+ 1 2 3) (- 1 2))",
                "(eq? (+ 1 2 3) 6 0)",
                "(and (or 'true 'false) 'false)"];
    let bad = ["(+ 1 2 3 (>= 5 7))",
               "(-)",
               "(xor 1)",
               "(+ x y z)", // unbound variables.
               "(+ 1 2 3 (eq? 1 2))",
               "(and (or 'true 'false) (+ 1 2 3))"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_simple_hash_checks() {
    let good = ["(hash160 1)",
                "(sha256 (keccak256 1))"];
    let bad_types = ["(hash160 'true)",
                     "(sha256 'false)",
                     "(keccak256 (list 1 2 3))"];
    let invalid_args = ["(sha256 1 2 3)"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad_types.iter() {
        assert!(match type_check_helper(&bad_test).unwrap_err().err {
            CheckErrors::UnionTypeError(_, _) => true,
            _ => false
        })
    }
    
    for bad_test in invalid_args.iter() {
        assert!(match type_check_helper(&bad_test).unwrap_err().err {
            CheckErrors::IncorrectArgumentCount(_, _) => true,
            _ => false
        })
    }
}

#[test]
fn test_simple_ifs() {
    let good = ["(if (> 1 2) (+ 1 2 3) (- 1 2))",
                "(if 'true 'true 'false)",
                "(if 'true \"abcdef\" \"abc\")",
                "(if 'true \"a\" \"abcdef\")" ];
    let bad = ["(if 'true 'true 1)",
               "(if 'true \"a\" 'false)",
               "(if)",
               "(if 0 1 0)"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_simple_lets() {
    let good = ["(let ((x 1) (y 2) (z 3)) (if (> x 2) (+ 1 x y) (- 1 z)))",
                "(let ((x 'true) (y (+ 1 2)) (z 3)) (if x (+ 1 z y) (- 1 z)))",
                "(let ((x 'true) (y (+ 1 2)) (z 3)) (print x) (if x (+ 1 z y) (- 1 z)))"];
    let bad = ["(let ((1)) (+ 1 2))",
               "(let ((1 2)) (+ 1 2))"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_eqs() {
    let good = ["(eq? (list 1 2 3 4 5) (list 1 2 3 4 5 6 7))",
                "(eq? (tuple (good 1) (bad 2)) (tuple (good 2) (bad 3)))",
                "(eq? \"abcdef\" \"abc\" \"a\")"];
    let bad = [
        "(eq? 1 2 'false)",
        "(eq? 1 2 3 (list 2))",
        "(eq? (some 1) (some 'true))",
        "(list (list 1 2) (list 'true) (list 5 1 7))",
        "(list 1 2 3 'true 'false 4 5 6)",
        "(map mod (list 1 2 3 4 5))",
        "(map - (list 'true 'false 'true 'false))",
        "(map hash160 (+ 1 2))",];

    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_lists() {
    let good = ["(map hash160 (list 1 2 3 4 5))",
                "(list (list 1 2) (list 3 4) (list 5 1 7))",
                "(filter not (list 'false 'true 'false))",
                "(fold and (list 'true 'true 'false 'false) 'true)",
                "(map - (list (+ 1 2) 3 (+ 4 5) (* (+ 1 2) 3)))"];
    let bad = [
        "(fold and (list 'true 'false) 2)",
        "(fold hash160 (list 1 2 3 4) 2)",
        "(fold >= (list 1 2 3 4) 2)",
        "(list (list 1 2) (list 'true) (list 5 1 7))",
        "(list 1 2 3 'true 'false 4 5 6)",
        "(filter hash160 (list 1 2 3 4))",
        "(filter not (list 1 2 3 4))",
        "(filter not (list 1 2 3 4) 1)",
        "(filter ynot (list 1 2 3 4) 1)",
        "(map if (list 1 2 3 4 5))",
        "(map mod (list 1 2 3 4 5))",
        "(map - (list 'true 'false 'true 'false))",
        "(map hash160 (+ 1 2))",];

    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_lists_in_defines() {
    let good = "
    (define-private (test (x int)) (eq? 0 (mod x 2)))
    (filter test (list 1 2 3 4 5))";
    mem_type_check(good).unwrap();
}

#[test]
fn test_tuples() {
    let good = ["(+ 1 2     (get abc (tuple (abc 1) (def 'true))))",
                "(and 'true (get def (tuple (abc 1) (def 'true))))"];
    let bad = ["(+ 1 2      (get def (tuple (abc 1) (def 'true))))",
               "(and 'true  (get abc (tuple (abc 1) (def 'true))))"];
    for good_test in good.iter() {
        type_check_helper(&good_test).unwrap();
    }
    
    for bad_test in bad.iter() {
        type_check_helper(&bad_test).unwrap_err();
    }
}

#[test]
fn test_empty_tuple_should_fail() {
    let contract_src = r#"
        (define-private (set-cursor (value (tuple)))
            value)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::BadSyntaxBinding => true,
        _ => false
    });
}

#[test]
fn test_define() {
    let good = ["(define-private (foo (x int) (y int)) (+ x y))
                     (define-private (bar (x int) (y bool)) (if y (+ 1 x) 0))
                     (* (foo 1 2) (bar 3 'false))",
    ];
    
    let bad = ["(define-private (foo ((x int) (y int)) (+ x y)))
                     (define-private (bar ((x int) (y bool)) (if y (+ 1 x) 0)))
                     (* (foo 1 2) (bar 3 3))",
    ];

    for good_test in good.iter() {
        mem_type_check(good_test).unwrap();
    }

    for bad_test in bad.iter() {
        mem_type_check(bad_test).unwrap_err();
    }
}

#[test]
fn test_function_arg_names() {
    use vm::analysis::type_check;
    
    let functions = vec![
        "(define-private (test (x int)) (ok 0))
         (define-public (test-pub (x int)) (ok 0))
         (define-read-only (test-ro (x int)) (ok 0))",

        "(define-private (test (x int) (y bool)) (ok 0))
         (define-public (test-pub (x int) (y bool)) (ok 0))
         (define-read-only (test-ro (x int) (y bool)) (ok 0))",

        "(define-private (test (name-1 int) (name-2 int) (name-3 int)) (ok 0))
         (define-public (test-pub (name-1 int) (name-2 int) (name-3 int)) (ok 0))
         (define-read-only (test-ro (name-1 int) (name-2 int) (name-3 int)) (ok 0))",

        "(define-private (test) (ok 0))
         (define-public (test-pub) (ok 0))
         (define-read-only (test-ro) (ok 0))",
    ];

    let expected_arg_names: Vec<Vec<&str>> = vec![
        vec!["x"],
        vec!["x", "y"],
        vec!["name-1", "name-2", "name-3"],
        vec![],
    ];

    for (func_test, arg_names) in functions.iter().zip(expected_arg_names.iter()) {
        let contract_analysis = mem_type_check(func_test).unwrap();

        let func_type_priv = contract_analysis.get_private_function("test").unwrap();
        let func_type_pub = contract_analysis.get_public_function_type("test-pub").unwrap();
        let func_type_ro = contract_analysis.get_read_only_function_type("test-ro").unwrap();

        for func_type in &[func_type_priv, func_type_pub, func_type_ro] {
            let func_args = match func_type {
                FunctionType::Fixed(args, _) => args,
                _ => panic!("Unexpected function type")
            };
            
            for (expected_name, actual_name) in arg_names.iter().zip(func_args.iter().map(|x| &x.name)) {
                assert_eq!(*expected_name, actual_name);
            }
        }
    }
}

#[test]
fn test_factorial() {
    let contract = "(define-map factorials ((id int)) ((current int) (index int)))
         (define-private (init-factorial (id int) (factorial int))
           (print (map-insert! factorials (tuple (id id)) (tuple (current 1) (index factorial)))))
         (define-public (compute (id int))
           (let ((entry (expects! (map-get factorials (tuple (id id)))
                                 (err 'false))))
                    (let ((current (get current entry))
                          (index   (get index entry)))
                         (if (<= index 1)
                             (ok 'true)
                             (begin
                               (map-set! factorials (tuple (id id))
                                                      (tuple (current (* current index))
                                                             (index (- index 1))))
                               (ok 'false))))))
        (begin (init-factorial 1337 3)
               (init-factorial 8008 5))
        ";

    mem_type_check(contract).unwrap();
}

#[test]
fn test_options() {
    let contract = "
         (define-private (foo (id (optional int)))
           (+ 1 (default-to 1 id)))
         (define-private (bar (x int))
           (if (> 0 x)
               (some x)
               none))
         (+ (foo none)
            (foo (bar 1))
            (foo (bar 0)))
         ";

    mem_type_check(contract).unwrap();

    let contract = "
         (define-private (foo (id (optional bool)))
           (if (default-to 'false id)
               1
               0))
         (define-private (bar (x int))
           (if (> 0 x)
               (some x)
               none))
         (+ (foo (bar 1)) 1)
         ";

    assert!(
        match mem_type_check(contract).unwrap_err().err {
            CheckErrors::TypeError(t1, t2) => {
                t1 == TypeSignature::Atom(AtomTypeIdentifier::OptionalType(
                    Box::new(TypeSignature::Atom(AtomTypeIdentifier::BoolType)))) &&
                t2 == TypeSignature::Atom(AtomTypeIdentifier::OptionalType(
                    Box::new(TypeSignature::Atom(AtomTypeIdentifier::IntType))))
            },
            _ => false
        });

}

#[test]
fn test_set_int_variable() {
    let contract_src = r#"
        (define-data-var cursor int 0)
        (define-private (get-cursor)
            (var-get cursor))
        (define-private (set-cursor (value int))
            (if (var-set! cursor value)
                value
                0))
        (define-private (increment-cursor)
            (begin
                (var-set! cursor (+ 1 (get-cursor)))
                (get-cursor)))
    "#;

    mem_type_check(contract_src).unwrap();
}

#[test]
fn test_set_bool_variable() {
    let contract_src = r#"
        (define-data-var is-ok bool 'true)
        (define-private (get-ok)
            (var-get is-ok))
        (define-private (set-cursor (new-ok bool))
            (if (var-set! is-ok new-ok)
                new-ok
                (get-ok)))
    "#;

    mem_type_check(contract_src).unwrap();
}

#[test]
fn test_set_tuple_variable() {
    let contract_src = r#"
        (define-data-var cursor (tuple (k1 int) (v1 int)) (tuple (k1 1) (v1 1)))
        (define-private (get-cursor)
            (var-get cursor))
        (define-private (set-cursor (value (tuple (k1 int) (v1 int))))
            (if (var-set! cursor value)
                value
                (get-cursor)))
    "#;

    mem_type_check(contract_src).unwrap();
}

#[test]
fn test_set_list_variable() {
    let contract_src = r#"
        (define-data-var ranking (list 3 int) (list 1 2 3))
        (define-private (get-ranking)
            (var-get ranking))
        (define-private (set-ranking (new-ranking (list 3 int)))
            (if (var-set! ranking new-ranking)
                new-ranking
                (get-ranking)))
    "#;

    mem_type_check(contract_src).unwrap();
}

#[test]
fn test_set_buffer_variable() {
    let contract_src = r#"
        (define-data-var name (buff 5) "alice")
        (define-private (get-name)
            (var-get name))
        (define-private (set-name (new-name (buff 3)))
            (if (var-set! name new-name)
                new-name
                (get-name)))
    "#;

    mem_type_check(contract_src).unwrap();
}

#[test]
fn test_missing_value_on_declaration_should_fail() {
    let contract_src = r#"
        (define-data-var cursor int)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::IncorrectArgumentCount(_, _) => true,
        _ => false
    });
}

#[test]
fn test_mismatching_type_on_declaration_should_fail() {
    let contract_src = r#"
        (define-data-var cursor int 'true)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::TypeError(_, _) => true,
        _ => false
    });
}

#[test]
fn test_mismatching_type_on_update_should_fail() {
    let contract_src = r#"
        (define-data-var cursor int 0)
        (define-private (get-cursor)
            (var-get cursor))
        (define-private (set-cursor (value principal))
            (if (var-set! cursor value)
                value
                0))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::TypeError(_, _) => true,
        _ => false
    });
}

#[test]
fn test_direct_access_to_persisted_var_should_fail() {
    let contract_src = r#"
        (define-data-var cursor int 0)
        (define-private (get-cursor)
            cursor)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::UnboundVariable(_) => true,
        _ => false
    });
}

#[test]
fn test_data_var_shadowed_by_let_should_fail() {
    let contract_src = r#"
        (define-data-var cursor int 0)
        (define-private (set-cursor (value int))
            (let ((cursor 0))
               (if (var-set! cursor value)
                   value
                    0)))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NameAlreadyUsed(_) => true,
        _ => false
    });
}

#[test]
fn test_mutating_unknown_data_var_should_fail() {
    let contract_src = r#"
        (define-private (set-cursor (value int))
            (if (var-set! cursor value)
                value
                0))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NoSuchVariable(_) => true,
        _ => false
    });
}

#[test]
fn test_accessing_unknown_data_var_should_fail() {
    let contract_src = r#"
        (define-private (get-cursor)
            (expects! (var-get cursor) 0))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NoSuchVariable(_) => true,
        _ => false
    });
}

#[test]
fn test_let_shadowed_by_let_should_fail() {
    let contract_src = r#"
        (let ((cursor 1) (cursor 2))
            cursor)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NameAlreadyUsed(_) => true,
        _ => false
    });
}

#[test]
fn test_let_shadowed_by_nested_let_should_fail() {
    let contract_src = r#"
        (let ((cursor 1))
            (let ((cursor 2))
                cursor))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NameAlreadyUsed(_) => true,
        _ => false
    });
}

#[test]
fn test_define_constant_shadowed_by_let_should_fail() {
    let contract_src = r#"
        (define-private (cursor) 0)
        (define-private (set-cursor (value int))
            (let ((cursor 1))
               cursor))
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NameAlreadyUsed(_) => true,
        _ => false
    });
}

#[test]
fn test_define_constant_shadowed_by_argument_should_fail() {
    let contract_src = r#"
        (define-private (cursor) 0)
        (define-private (set-cursor (cursor int))
            cursor)
    "#;

    let res = mem_type_check(contract_src).unwrap_err();
    assert!(match &res.err {
        &CheckErrors::NameAlreadyUsed(_) => true,
        _ => false
    });
}

#[test]
fn test_tuple_map() {
    let t = "(define-map tuples ((name int)) 
                            ((contents (tuple (name (buff 5))
                                              (owner (buff 5))))))

         (define-private (add-tuple (name int) (content (buff 5)))
           (map-insert! tuples (tuple (name name))
                                 (tuple (contents
                                   (tuple (name content)
                                          (owner content))))))
         (define-private (get-tuple (name int))
            (get name (get contents (map-get tuples (tuple (name name))))))


         (add-tuple 0 \"abcde\")
         (add-tuple 1 \"abcd\")
         (list      (get-tuple 0)
                    (get-tuple 1))
        ";
    mem_type_check(t).unwrap();
}


#[test]
fn test_explicit_tuple_map() {
    let contract =
        "(define-map kv-store ((key int)) ((value int)))
          (define-private (kv-add (key int) (value int))
             (begin
                 (map-insert! kv-store (tuple (key key))
                                     (tuple (value value)))
             value))
          (define-private (kv-get (key int))
             (expects! (get value (map-get kv-store (tuple (key key)))) 0))
          (define-private (kv-set (key int) (value int))
             (begin
                 (map-set! kv-store (tuple (key key))
                                    (tuple (value value)))
                 value))
          (define-private (kv-del (key int))
             (begin
                 (map-delete! kv-store (tuple (key key)))
                 key))
         ";

    mem_type_check(contract).unwrap();
}

#[test]
fn test_implicit_tuple_map() {
    let contract =
         "(define-map kv-store ((key int)) ((value int)))
          (define-private (kv-add (key int) (value int))
             (begin
                 (map-insert! kv-store ((key key))
                                     ((value value)))
             value))
          (define-private (kv-get (key int))
             (expects! (get value (map-get kv-store ((key key)))) 0))
          (define-private (kv-set (key int) (value int))
             (begin
                 (map-set! kv-store ((key key))
                                    ((value value)))
                 value))
          (define-private (kv-del (key int))
             (begin
                 (map-delete! kv-store ((key key)))
                 key))
         ";

    mem_type_check(contract).unwrap();
}


#[test]
fn test_bound_tuple_map() {
    let contract =
        "(define-map kv-store ((key int)) ((value int)))
         (define-private (kv-add (key int) (value int))
            (begin
                (let ((my-tuple (tuple (key key))))
                (map-insert! kv-store (tuple (key key))
                                    (tuple (value value))))
            value))
         (define-private (kv-get (key int))
            (let ((my-tuple (tuple (key key))))
            (expects! (get value (map-get kv-store my-tuple)) 0)))
         (define-private (kv-set (key int) (value int))
            (begin
                (let ((my-tuple (tuple (key key))))
                (map-set! kv-store my-tuple
                                   (tuple (value value))))
                value))
         (define-private (kv-del (key int))
            (begin
                (let ((my-tuple (tuple (key key))))
                (map-delete! kv-store my-tuple))
                key))
        ";

    mem_type_check(contract).unwrap();
}

#[test]
fn test_fetch_entry_matching_type_signatures() {
    let cases = [
        "map-get kv-store ((key key))",
        "map-get kv-store ((key 0))",
        "map-get kv-store (tuple (key 0))",
        "map-get kv-store (compatible-tuple)",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (compatible-tuple) (tuple (key 1)))
             (define-private (kv-get (key int))
                ({}))", case);

        mem_type_check(&contract_src).unwrap();
    }
}

#[test]
fn test_fetch_entry_mismatching_type_signatures() {
    let cases = [
        "map-get kv-store ((incomptible-key key))",
        "map-get kv-store ((key 'true))",
        "map-get kv-store (incompatible-tuple)",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (incompatible-tuple) (tuple (k 1)))
             (define-private (kv-get (key int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::TypeError(_, _) => true,
            _ => false
        });
    }
}

#[test]
fn test_fetch_entry_unbound_variables() {
    let cases = [
        "map-get kv-store ((key unknown-value))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (kv-get (key int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::UnboundVariable(_) => true,
            _ => false
        });
    }
}

#[test]
fn test_insert_entry_matching_type_signatures() {
    let cases = [
        "map-insert! kv-store ((key key)) ((value value))",
        "map-insert! kv-store ((key 0)) ((value 1))",
        "map-insert! kv-store (tuple (key 0)) (tuple (value 1))",
        "map-insert! kv-store (compatible-tuple) ((value 1))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (compatible-tuple) (tuple (key 1)))
             (define-private (kv-add (key int) (value int))
                ({}))", case);
        mem_type_check(&contract_src).unwrap();
    }
}

#[test]
fn test_insert_entry_mismatching_type_signatures() {
    let cases = [
        "map-insert! kv-store ((incomptible-key key)) ((value value))",
        "map-insert! kv-store ((key key)) ((incomptible-key value))",
        "map-insert! kv-store ((key 'true)) ((value 1))",
        "map-insert! kv-store ((key key)) ((value 'true))",
        "map-insert! kv-store (incompatible-tuple) ((value 1))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (incompatible-tuple) (tuple (k 1)))
             (define-private (kv-add (key int) (value int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::TypeError(_, _) => true,
            _ => false
        });
    }
}

#[test]
fn test_insert_entry_unbound_variables() {
    let cases = [
        "map-insert! kv-store ((key unknown-value)) ((value 1))",
        "map-insert! kv-store ((key key)) ((value unknown-value))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (kv-add (key int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::UnboundVariable(_) => true,
            _ => false
        });
    }
}


#[test]
fn test_delete_entry_matching_type_signatures() {
    let cases = [
        "map-delete! kv-store ((key key))",
        "map-delete! kv-store ((key 1))",
        "map-delete! kv-store (tuple (key 1))",
        "map-delete! kv-store (compatible-tuple)",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (compatible-tuple) (tuple (key 1)))
             (define-private (kv-del (key int))
                ({}))", case);
        mem_type_check(&contract_src).unwrap();
    }
}

#[test]
fn test_delete_entry_mismatching_type_signatures() {
    let cases = [
        "map-delete! kv-store ((incomptible-key key))",
        "map-delete! kv-store ((key 'true))",
        "map-delete! kv-store (incompatible-tuple)",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (incompatible-tuple) (tuple (k 1)))
             (define-private (kv-del (key int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::TypeError(_, _) => true,
            _ => false
        });
    }

}

#[test]
fn test_delete_entry_unbound_variables() {    
    let cases = [
        "map-delete! kv-store ((key unknown-value))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (kv-del (key int))
                ({}))", case);
        let res = mem_type_check(&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::UnboundVariable(_) => true,
            _ => false
        });
    }
}

#[test]
fn test_set_entry_matching_type_signatures() {    
    let cases = [
        "map-set! kv-store ((key key)) ((value value))",
        "map-set! kv-store ((key 0)) ((value 1))",
        "map-set! kv-store (tuple (key 0)) (tuple (value 1))",
        "map-set! kv-store (tuple (key 0)) (tuple (value known-value))",
        "map-set! kv-store (compatible-tuple) ((value 1))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (compatible-tuple) (tuple (key 1)))
             (define-private (kv-set (key int) (value int))
                (let ((known-value 2))
                ({})))", case);
        mem_type_check(&contract_src).unwrap();
    }
}



#[test]
fn test_set_entry_mismatching_type_signatures() {    
    let cases = [
        "map-set! kv-store ((incomptible-key key)) ((value value))",
        "map-set! kv-store ((key key)) ((incomptible-key value))",
        "map-set! kv-store ((key 'true)) ((value 1))",
        "map-set! kv-store ((key key)) ((value 'true))",
        "map-set! kv-store (incompatible-tuple) ((value 1))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (incompatible-tuple) (tuple (k 1)))
             (define-private (kv-set (key int) (value int))
                ({}))", case);
        let res = mem_type_check(&&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::TypeError(_, _) => true,
            _ => false
        });
    }
}


#[test]
fn test_set_entry_unbound_variables() {    
    let cases = [
        "map-set! kv-store ((key unknown-value)) ((value 1))",
        "map-set! kv-store ((key key)) ((value unknown-value))",
    ];

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (kv-set (key int) (value int))
                ({}))", case);
        let res = mem_type_check(&&contract_src).unwrap_err();
        assert!(match &res.err {
            &CheckErrors::UnboundVariable(_) => true,
            _ => false
        });
    }
}

#[test]
fn test_fetch_contract_entry_matching_type_signatures() {    
    let kv_store_contract_src = r#"
        (define-map kv-store ((key int)) ((value int)))
        (define-read-only (kv-get (key int))
            (expects! (get value (map-get kv-store ((key key)))) 0))
        (begin (map-insert! kv-store ((key 42)) ((value 42))))"#;

    let mut analysis_db = AnalysisDatabase::memory();

    let contract_id = QualifiedContractIdentifier::local("kv-store-contract").unwrap();

    let mut kv_store_contract = parse(&contract_id, &kv_store_contract_src).unwrap();
    analysis_db.execute(|db| {
        type_check(&contract_id, &mut kv_store_contract, db, true)
    }).unwrap();

    let cases = [
        "contract-map-get kv-store-contract kv-store ((key key))",
        "contract-map-get kv-store-contract kv-store ((key 0))",
        "contract-map-get kv-store-contract kv-store (tuple (key 0))",
        "contract-map-get kv-store-contract kv-store (compatible-tuple)",
    ];

    let transient_contract_id = QualifiedContractIdentifier::transient();

    for case in cases.into_iter() {
        let contract_src = format!(r#"
            (define-private (compatible-tuple) (tuple (key 1)))
            (define-private (kv-get (key int)) ({}))"#, case);
        let mut contract = parse(&transient_contract_id, &contract_src).unwrap();
        analysis_db.execute(|db| {
            type_check(&transient_contract_id, &mut contract, db, false)
        }).unwrap();
    }
}

#[test]
fn test_fetch_contract_entry_mismatching_type_signatures() {
    let kv_store_contract_src = r#"
        (define-map kv-store ((key int)) ((value int)))
        (define-read-only (kv-get (key int))
            (expects! (get value (map-get kv-store ((key key)))) 0))
        (begin (map-insert! kv-store ((key 42)) ((value 42))))"#;

    let contract_id = QualifiedContractIdentifier::local("kv-store-contract").unwrap();
    let mut analysis_db = AnalysisDatabase::memory();
    let mut kv_store_contract = parse(&contract_id, &kv_store_contract_src).unwrap();
    analysis_db.execute(|db| {
        type_check(&contract_id, &mut kv_store_contract, db, true)
    }).unwrap();
    
    let cases = [
        "contract-map-get kv-store-contract kv-store ((incomptible-key key))",
        "contract-map-get kv-store-contract kv-store ((key 'true))",
        "contract-map-get kv-store-contract kv-store (incompatible-tuple)",
    ];

    let transient_contract_id = QualifiedContractIdentifier::transient();

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (incompatible-tuple) (tuple (k 1)))
             (define-private (kv-get (key int))
                ({}))", case);
        let mut contract = parse(&transient_contract_id, &contract_src).unwrap();
        let res = 
            analysis_db.execute(|db| {
                type_check(&transient_contract_id, &mut contract, db, false)
            }).unwrap_err();

        assert!(match &res.err {
            &CheckErrors::TypeError(_, _) => true,
            _ => false
        });
    }
}

#[test]
fn test_fetch_contract_entry_unbound_variables() {
    let kv_store_contract_src = r#"
        (define-map kv-store ((key int)) ((value int)))
        (define-read-only (kv-get (key int))
            (expects! (get value (map-get kv-store ((key key)))) 0))
        (begin (map-insert! kv-store ((key 42)) ((value 42))))"#;

    let contract_id = QualifiedContractIdentifier::local("kv-store-contract").unwrap();
    let mut analysis_db = AnalysisDatabase::memory();
    let mut kv_store_contract = parse(&contract_id, &kv_store_contract_src).unwrap();
    analysis_db.execute(|db| {
        type_check(&contract_id, &mut kv_store_contract, db, true)
    }).unwrap();
    
    let cases = [
        "contract-map-get kv-store-contract kv-store ((key unknown-value))",
    ];

    let transient_contract_id = QualifiedContractIdentifier::transient();

    for case in cases.into_iter() {
        let contract_src = format!(
            "(define-map kv-store ((key int)) ((value int)))
             (define-private (kv-get (key int))
                ({}))", case);
        let mut contract = parse(&transient_contract_id, &contract_src).unwrap();
        let res = 
            analysis_db.execute(|db| {
                type_check(&transient_contract_id, &mut contract, db, false)
            }).unwrap_err();

        assert!(match &res.err {
            &CheckErrors::UnboundVariable(_) => true,
            _ => false
        });
    }
}
