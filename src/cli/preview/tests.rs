//! Tests for the `lazyqmk preview` CLI subcommand.

use super::*;

// -- parse_highlight tests ---------------------------------------------------

#[test]
fn test_parse_highlight_simple() {
    let ((row, col), ch) = parse_highlight("(0,2)=B").unwrap();
    assert_eq!((row, col), (0, 2));
    assert_eq!(ch, 'B');
}

#[test]
fn test_parse_highlight_with_spaces() {
    let ((row, col), ch) = parse_highlight("  ( 1 , 3 ) = ★  ").unwrap();
    assert_eq!((row, col), (1, 3));
    assert_eq!(ch, '★');
}

#[test]
fn test_parse_highlight_unicode_marker() {
    let ((row, col), ch) = parse_highlight("(2,4)=→").unwrap();
    assert_eq!((row, col), (2, 4));
    assert_eq!(ch, '→');
}

#[test]
fn test_parse_highlight_rejects_missing_equals() {
    let err = parse_highlight("(0,2)").unwrap_err();
    assert!(err.contains("(row,col)=CHAR"), "unexpected error: {err}");
}

#[test]
fn test_parse_highlight_rejects_missing_parens() {
    let err = parse_highlight("0,2=B").unwrap_err();
    assert!(err.contains("(row,col)"), "unexpected error: {err}");
}

#[test]
fn test_parse_highlight_rejects_multi_char_marker() {
    let err = parse_highlight("(0,2)=BC").unwrap_err();
    assert!(err.contains("single character"), "unexpected error: {err}");
}

#[test]
fn test_parse_highlight_rejects_empty_marker() {
    let err = parse_highlight("(0,2)=").unwrap_err();
    assert!(err.contains("empty") || err.contains("single"), "unexpected error: {err}");
}

#[test]
fn test_parse_highlight_rejects_non_numeric_row() {
    let err = parse_highlight("(abc,2)=B").unwrap_err();
    assert!(err.contains("row"), "unexpected error: {err}");
}

#[test]
fn test_parse_highlight_rejects_missing_comma() {
    let err = parse_highlight("(02)=B").unwrap_err();
    assert!(err.contains("row,col"), "unexpected error: {err}");
}
