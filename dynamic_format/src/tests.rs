use crate::{Formatting, Piece, Placeholder};

#[test]
fn parse() {
    // Cases from https://learn.microsoft.com/en-us/dotnet/standard/base-types/composite-formatting
    let cases = [
        (
            Ok(Formatting {
                pieces: vec![
                    Piece::Raw("Name = "),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "",
                    }),
                    Piece::Raw(", hours = "),
                    Piece::Placeholder(Placeholder {
                        index: 1,
                        width: None,
                        fmt_string: "hh",
                    }),
                ],
            }),
            "Name = {0}, hours = {1:hh}",
        ),
        (
            Ok(Formatting {
                pieces: vec![
                    Piece::Raw("Four prime numbers: "),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "",
                    }),
                    Piece::Raw(", "),
                    Piece::Placeholder(Placeholder {
                        index: 1,
                        width: None,
                        fmt_string: "",
                    }),
                    Piece::Raw(", "),
                    Piece::Placeholder(Placeholder {
                        index: 2,
                        width: None,
                        fmt_string: "",
                    }),
                    Piece::Raw(", "),
                    Piece::Placeholder(Placeholder {
                        index: 3,
                        width: None,
                        fmt_string: "",
                    }),
                ],
            }),
            "Four prime numbers: {0}, {1}, {2}, {3}",
        ),
        (
            Ok(Formatting {
                pieces: vec![
                    Piece::Raw("0x"),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "X",
                    }),
                    Piece::Raw(" "),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "E",
                    }),
                    Piece::Raw(" "),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "N",
                    }),
                ],
            }),
            "0x{0:X} {0:E} {0:N}",
        ),
        (
            Ok(Formatting {
                pieces: vec![
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: Some(-20),
                        fmt_string: "",
                    }),
                    Piece::Raw(" "),
                    Piece::Placeholder(Placeholder {
                        index: 1,
                        width: Some(5),
                        fmt_string: "N1",
                    }),
                ],
            }),
            "{0,-20} {1,5:N1}",
        ),
        // Parsed as https://learn.microsoft.com/en-us/dotnet/standard/base-types/composite-formatting#net
        (
            Ok(Formatting {
                pieces: vec![
                    Piece::Raw("{"),
                    Piece::Placeholder(Placeholder {
                        index: 0,
                        width: None,
                        fmt_string: "D",
                    }),
                    Piece::Raw("}"),
                ],
            }),
            "{{{0:D}}}",
        ),
    ];

    for (expect, case) in cases {
        assert_eq!(expect, Formatting::parse(case));
    }
}
