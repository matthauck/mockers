#![feature(plugin, custom_derive)]
#![plugin(mockers_macros)]

extern crate mockers;

use std::rc::Rc;
use std::panic::AssertUnwindSafe;

use mockers::{Scenario, Sequence};
use mockers::matchers::{ANY, lt};

#[derive(Mock)]
pub trait A {
    fn foo(&self);
    fn bar(&self, arg: u32);
    fn baz(&self) -> u32;
    fn modify(&mut self);
    fn ask(&self, arg: u32) -> u32;
    fn consume(self);
    fn consume_result(&self) -> String;
    fn consume_arg(&self, arg: String) -> String;
    fn consume_rc(&self, arg: Rc<usize>);
}

mock!{
    AMockByMacro,
    self,
    trait A {
        fn foo(&self);
        fn bar(&self, arg: u32);
        fn baz(&self) -> u32;
        fn modify(&mut self);
        fn ask(&self, arg: u32) -> u32;
        fn consume(self);
        fn consume_result(&self) -> String;
        fn consume_arg(&self, arg: String) -> String;
        fn consume_rc(&self, arg: Rc<usize>);
    }
}

#[test]
#[should_panic(expected="unexpected call to `A#0.foo()`")]
fn test_unit() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.bar_call(2).and_return(()));
    mock.foo();
}

#[test]
fn test_return() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.baz_call().and_return(2));
    assert_eq!(2, mock.baz());
}


#[test]
#[should_panic(expected="4 is not less than 3")]
fn test_arg_match_failure() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.bar_call(lt(3)).and_return(()));
    mock.bar(4);
}

#[test]
fn test_arg_match_success() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.bar_call(lt(3)).and_return(()));
    mock.bar(2);
}


#[test]
#[should_panic(expected="Some expectations are not satisfied:\n`A#0.bar(_)`\n")]
fn test_expected_call_not_performed() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.bar_call(ANY).and_return(()));
}


#[test]
#[should_panic(expected="boom!")]
fn test_panic_result() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.foo_call().and_panic("boom!".to_owned()));
    mock.foo();
}

#[test]
fn test_mut_self_method() {
    let mut scenario = Scenario::new();
    let mut mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.modify_call().and_return(()));
    mock.modify();
}

#[test]
fn test_value_self_method() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();
    scenario.expect(mock.consume_call().and_return(()));
    mock.consume();
}

#[test]
#[should_panic(expected="unexpected call to `amock.foo()`")]
fn test_named_mock() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_named_mock_for::<A>("amock".to_owned());
    scenario.expect(mock.bar_call(2).and_return(()));
    mock.foo();
}

/// Test that when test is failed, then remaining scenario
/// expectations are not checked and don't cause panic-during-drop
/// which will lead to ugly failure with not very useful message.
#[test]
#[should_panic(expected="caboom!")]
fn test_failed_with_remaining_expectations() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    // This expectation will never be satisfied.
    scenario.expect(mock.bar_call(2).and_return(()));
    panic!("caboom!");
}

#[test]
fn test_expect_and_call() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    // This expectation will never be satisfied.
    scenario.expect(mock.ask_call(2).and_call(|arg| { arg+1 }));
    assert_eq!(mock.ask(2), 3);
}

#[test]
fn test_expect_is_unordered() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.foo_call().and_return(()));
    scenario.expect(mock.bar_call(2).and_return(()));

    mock.bar(2);
    mock.foo();
}

#[test]
#[should_panic(expected="A#0.foo was already called earlier")]
fn test_expect_consumes_one_call_only() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.foo_call().and_return(()));

    mock.foo();
    mock.foo();
}

#[test]
fn test_never_satisfied() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.foo_call().never());
}

#[test]
#[should_panic(expected="A#0.foo should never be called")]
fn test_never_not_satisfied() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.foo_call().never());

    mock.foo();
}

#[test]
fn test_consume_result() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let result = "ho-ho".to_owned();
    scenario.expect(mock.consume_result_call().and_return(result));

    assert_eq!(mock.consume_result(), "ho-ho");
}

#[test]
fn test_consume_call_result() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let result = "ho-ho".to_owned();
    scenario.expect(mock.consume_result_call().and_call(move || { result }));

    assert_eq!(mock.consume_result(), "ho-ho");
}

#[test]
fn test_consume_argument() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.consume_arg_call(ANY).and_call(|arg| { arg }));

    let arg = "ho-ho".to_owned();
    assert_eq!(mock.consume_arg(arg), "ho-ho");
}

#[test]
fn test_arguments_are_dropped_on_panic() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let arg = Rc::new(0);
    let weak = Rc::downgrade(&arg);
    assert!(weak.upgrade().is_some());

    let mock_ref = AssertUnwindSafe(&mock);
    let result = std::panic::catch_unwind(|| {
        // This will cause panic, because there is no matching
        // expectation. Argument must be dropped during unwinding.
        mock_ref.consume_rc(arg);
    });
    assert!(result.is_err());
    assert!(weak.upgrade().is_none());
}

#[test]
fn test_times_satisfied() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.baz_call().and_return_clone(4).times(2));

    mock.baz();
    mock.baz();
}

#[test]
#[should_panic(expected="Some expectations are not satisfied:
`A#0.baz() must be called 2 times, called 1 times`
")]
fn test_times_not_satisfied_less() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.baz_call().and_return_clone(4).times(2));

    mock.baz();
}

#[test]
#[should_panic(expected="A#0.baz was already called 2 times of 2 expected, extra call is unexpected")]
fn test_times_not_satisfied_more() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.baz_call().and_return_clone(4).times(2));

    mock.baz();
    mock.baz();
    mock.baz();
}

#[test]
#[should_panic(expected="`A#0.foo() must be called 2 times, called 1 times`")]
fn test_checkpoint() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.foo_call().and_return_clone(()).times(2));

    mock.foo();

    scenario.checkpoint();

    mock.foo();
}

#[test]
fn test_create_mock() {
    let mut scenario = Scenario::new();
    let _mock = scenario.create_mock::<AMockByMacro>();
}

#[test]
#[should_panic(expected="unexpected call to `A#0.bar(12)`")]
fn test_format_args() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    mock.bar(12);
}

// When no matching expectation found for call, expectations
// for other mock object of the same type must be checked.
#[test]
// Message without ANSI codes is "expectation `A#0.bar(12)`"
#[should_panic(expected="expectation `\x1b[1mA#0\x1b[0m.bar(12)`")]
fn test_check_other_mock_object_expectations() {
    let mut scenario = Scenario::new();
    let mock0 = scenario.create_mock_for::<A>();
    let mock1 = scenario.create_mock_for::<A>();

    scenario.expect(mock0.bar_call(12).and_return(()));

    mock1.bar(12);
}

#[test]
fn test_sequence() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let mut seq = Sequence::new();
    seq.expect(mock.foo_call().and_return(()));
    seq.expect(mock.bar_call(4).and_return(()));
    scenario.expect(seq);

    mock.foo();
    mock.bar(4);
}

#[test]
#[should_panic(expected="unexpected call to `A#0.bar(4)`")]
fn test_sequence_invalid_order() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let mut seq = Sequence::new();
    seq.expect(mock.foo_call().and_return(()));
    seq.expect(mock.bar_call(4).and_return(()));
    scenario.expect(seq);

    mock.bar(4);
    mock.foo();
}

#[test]
fn test_sequence_times() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let mut seq = Sequence::new();
    seq.expect(mock.foo_call().and_return_clone(()).times(2));
    seq.expect(mock.bar_call(4).and_return(()));
    scenario.expect(seq);

    mock.foo();
    mock.foo();
    mock.bar(4);
}

#[test]
#[should_panic(expected="unexpected call to `A#0.bar(4)`")]
fn test_sequence_times_invalid() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    let mut seq = Sequence::new();
    seq.expect(mock.foo_call().and_return_clone(()).times(2));
    seq.expect(mock.bar_call(4).and_return(()));
    scenario.expect(seq);

    mock.foo();
    mock.bar(4);
}

#[test]
fn test_return_default() {
    let mut scenario = Scenario::new();
    let mock = scenario.create_mock_for::<A>();

    scenario.expect(mock.baz_call().and_return_default().times(1));

    assert_eq!(mock.baz(), 0);
}
