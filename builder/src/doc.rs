use crate::op::OpCode;
use strum::IntoEnumIterator;

pub struct OpDoc {
    flag: &'static str,
    args: &'static str,
    short: &'static str,
}

impl OpCode {
    pub fn doc(&self) -> OpDoc {
        match self {
            OpCode::Empty => OpDoc {
                flag: "--empty",
                args: "",
                short: "Push an empty archive to the stack.",
            },
            OpCode::Import => OpDoc {
                flag: "--import",
                args: " base [target...]",
                short: "Copy directories into the DB as archives.",
            },
        }
    }
}

pub fn usage() -> String {
    let mut sections: Vec<&str> = vec![];
    sections.push("Usage: dirtabase [op...]\n\n");
    sections.push("Valid ops:\n\n");

    for oc in OpCode::iter() {
        let doc = oc.doc();
        sections.extend([
            doc.flag,
            ": ",
            doc.short,
            "\n    Usage: ",
            doc.flag,
            doc.args,
            "\n\n",
        ]);
    }
    sections.concat()
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_usage() {
        assert!(
            usage().starts_with(indoc! {"
            Usage: dirtabase [op...]

            Valid ops:

            --empty: Push an empty archive to the stack.
                Usage: --empty

            --import: Copy directories into the DB as archives.
                Usage: --import base [target...]
        "}),
            "Got: {:?}",
            usage()
        )
    }
}
