use crate::gate::command::run_task;
use std::env;
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod gate {
    pub(crate) mod command {
        pub(crate) use gate_sync_command::run_task;
        mod gate_sync_command;
    }
    pub(crate) mod model {
        pub(crate) use document_block::DocumentBlock;
        pub(crate) use example_entry::ExampleEntry;
        pub(crate) use example_entry::ExampleFragment;
        pub(crate) use example_entry::Translation;
        pub(crate) use examples_file::ExamplesFile;
        pub(crate) use gate_arguments::GateArguments;
        pub(crate) use gate_error::GateError;
        pub(crate) use sync_change::SyncChange;
        pub(crate) use sync_result::SyncResult;
        mod document_block;
        mod example_entry;
        mod examples_file;
        mod gate_arguments;
        mod gate_error;
        mod sync_change;
        mod sync_result;
    }
    pub(crate) mod util {
        pub(crate) use document_names::collect_document_names;
        pub(crate) use example_hash::compute_example_hash;
        pub(crate) use example_hash::is_marker_line;
        pub(crate) use example_sync::sync_examples;
        pub(crate) use examples_parse::parse_examples;
        pub(crate) use examples_rewrite::rewrite_examples;
        pub(crate) use gate_argument_parse::parse_gate_arguments;
        pub(crate) use markdown_block::extract_blocks;
        mod document_names;
        mod example_hash;
        mod example_sync;
        mod examples_parse;
        mod examples_rewrite;
        mod gate_argument_parse;
        mod markdown_block;
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    run_task(&arguments, &workspace_dir)
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod main_test;
    mod gate {
        mod command {
            mod gate_sync_command_test;
        }
        mod util {
            mod document_names_test;
            mod example_hash_test;
            mod example_sync_test;
            mod examples_parse_test;
            mod examples_rewrite_test;
            mod gate_argument_parse_test;
            mod markdown_block_test;
        }
    }
}
