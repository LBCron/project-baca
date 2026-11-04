use web::App;

fn main() {
    console_error_panic_hook::set_once();
    // The static home shell from index.html stands in until now; the app
    // draws the same heading in the same place.
    if let Some(shell) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("boot-shell"))
    {
        shell.remove();
    }
    leptos::mount::mount_to_body(App);
}
