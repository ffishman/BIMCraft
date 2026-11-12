## Manage > Run Script: a script tried first and run on the open project as one undo step

The command line and agents can already run a script of commands (`id {json params}` one a line, `#` a comment), and since the dry run (#287) they can see what it would do before it does it. The desktop had no way to do the same: a user with a script (from a colleague, an agent's answer, a forum post) had to run it from the CLI on a file, outside the open project.

This adds **Manage > Macros > Run Script**, a dialog where the script is typed or pasted:

- **Try** is the core's dry run (`Session::script_dry_run`): one line saying how many elements it would make, change and remove, whether it changes the project's settings or library too, and how many warnings the model would then have. The project and its undo history are untouched.
- **Run** takes the same dry run first, so it refuses exactly what the dry run refuses (a line that names a file, saves, undoes, redoes or replaces the project), and then runs the script with `Session::script_atomic`: all or nothing, as one undo step labelled `Script`. The dialog closes and the status box says what it did.
- A refusal or an error is shown in the dialog, in the error colour, naming the line, and nothing has run.

The UI stays thin: everything goes through `Session`, no new command and no change to the core.

### Tests

- `run_script_tries_a_script_then_runs_it_as_one_undo_step_and_never_touches_a_file` (egui_kittest): Try leaves the model as it was, Run makes the wall as one `Script` undo step, and a script with a `save` line is refused naming line 2, with no file written.
- `cargo test --workspace`, clippy and `cargo i18n check` pass; the 11 new labels have Portuguese and Spanish.

### Docs

A paragraph in `docs/areas/desktop.md` next to the dry run's, and `script.rs` in the ui-egui module map.

Next, if this is welcome: recording a macro (the commands the window runs, written out as such a script), to be discussed first.
