use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Button, CssProvider, Image, Orientation, Overlay, Paned, 
    ScrolledWindow, TextView, TextBuffer, WrapMode, Align, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use glib::{self, timeout_add_local};
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use rand::Rng;

// import markdown.rs as a module
mod markdown;
use crate::markdown::markdown_to_html;

struct MarkdownEditor {
    window: ApplicationWindow,
    md_buffer: TextBuffer,
    html_buffer: TextBuffer,
    update_pending: Rc<RefCell<bool>>,
    html_template: String,
    markdown_chars: HashSet<char>,
    //is_dark_mode: Rc<RefCell<bool>>,
    //css_provider: CssProvider,
}

const LIGHT_MODE_CSS:&str = "
window {
    background-color: #f0e6d2; /* Warm cream background */
}

textview#static_html { 
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    font-size: 14px;
    background-color: #e8dcc3; /* Soft beige background */
    border-radius: 8px;
    padding: 20px;
    margin: 10px;
    color: #5c4b3a; /* Warm brown text */
    border: 1px solid #d4c5a8; /* Light beige border */
    box-shadow: 0 2px 8px rgba(160, 120, 80, 0.15); /* Soft amber shadow */
}

textview#static_html:focus {
    box-shadow: 0 2px 12px rgba(160, 120, 80, 0.25); /* Enhanced shadow on focus */
    border-color: #c0a080; /* Darker border on focus */
}

textview#static_html text {
    background-color: transparent;
}

textview#markdown_input {
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    font-size: 14px;
    background-color: #e8dcc3; /* Soft beige background */
    border-radius: 8px;
    padding: 20px;
    margin: 10px;
    color: #5c4b3a; /* Warm brown text */
    border: 1px solid #d4c5a8; /* Light beige border */
    box-shadow: 0 2px 8px rgba(160, 120, 80, 0.15); /* Soft amber shadow */
}

textview#markdown_input:focus {
    box-shadow: 0 2px 12px rgba(160, 120, 80, 0.25); /* Enhanced shadow on focus */
    border-color: #c0a080; /* Darker border on focus */
}

textview#markdown_input text {
    background-color: transparent;
}

button#theme_toggle {
    background-color: #d4c5a8;
    border-radius: 50%;
    border: none;
    padding: 8px;
    margin: 8px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
}

button#theme_toggle:hover {
    background-color: #c0a080;
}
";

const DARK_MODE_CSS:&str = "
window {
    background-color: #0c0c0c; /* D33p bl4ck b4ckgr0und */
}

textview#static_html { 
    font-family: 'Courier New', monospace;
    font-size: 14px;
    background-color: #000000; /* Pur3 bl4ck b4ckgr0und */
    border-radius: 0px; /* Sh4rp 3dg3s f0r h4x0r l00k */
    padding: 20px;
    margin: 10px;
    color: #ffffff; /* Wh1t3 t3xt f0r d4rk m0d3 */
    border: 1px solid #00ff00; /* N30n gr33n b0rd3r */
    box-shadow: 0 0 10px rgba(0, 255, 0, 0.5), 
                inset 0 0 5px rgba(0, 255, 0, 0.2); /* N30n gr33n gl0w */
    caret-color: #00ff00; /* Gr33n curs0r */
}

textview#static_html:focus {
    box-shadow: 0 0 15px rgba(0, 255, 0, 0.7), 
                inset 0 0 8px rgba(0, 255, 0, 0.3); /* 3nh4nc3d gl0w 0n f0cus */
    border: 1px solid #00ff00; /* Br1ght3r gr33n 0n f0cus */
}

textview#static_html text {
    background-color: transparent;
}

textview#markdown_input {
    font-family: 'Courier New', monospace;
    font-size: 14px;
    background-color: #000000; /* Pur3 bl4ck b4ckgr0und */
    border-radius: 0px; /* Sh4rp 3dg3s f0r h4x0r l00k */
    padding: 20px;
    margin: 10px;
    color: #00ff00; /* M4tr1x gr33n t3xt */
    border: 1px solid #00ff00; /* N30n gr33n b0rd3r */
    box-shadow: 0 0 10px rgba(0, 255, 0, 0.5), 
                inset 0 0 5px rgba(0, 255, 0, 0.2); /* N30n gr33n gl0w */
    caret-color: #00ff00; /* Gr33n curs0r */

}

textview#markdown_input:focus {
    box-shadow: 0 0 15px rgba(0, 255, 0, 0.7), 
                inset 0 0 8px rgba(0, 255, 0, 0.3); /* 3nh4nc3d gl0w 0n f0cus */
    border: 1px solid #00ff00; /* Br1ght3r gr33n 0n f0cus */
}

textview#markdown_input text {
    background-color: transparent;
    caret-color: #00ff00; /* Gr33n curs0r */
}

textview#markdown_input text selection {
    background-color: rgba(0, 255, 0, 0.3); /* Gr33n s3l3ct10n */
    color: #ffffff; /* Wh1t3 t3xt f0r s3l3ct3d t3xt */
}

button#theme_toggle {
    background-color: #000000;
    border: 1px solid #00ff00;
    border-radius: 0px; /* Sh4rp 3dg3s f0r h4x0r l00k */
    min-width: 36px;
    min-height: 36px;
    padding: 8px;
    margin: 8px;
    box-shadow: 0 0 8px rgba(0, 255, 0, 0.5);
}

button#theme_toggle:hover {
    background-color: #003300;
    box-shadow: 0 0 12px rgba(0, 255, 0, 0.8);
}
";

const THEME_BUTTON_CSS:&str = "
button#theme_toggle {
    background-color: #d4c5a8;
    border-radius: 50%;
    border: none;
    min-width: 36px;
    min-height: 36px;
    padding: 8px;
    margin: 8px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
    overflow: hidden; /* Ensure content doesn't overflow the circular shape */
}

button#theme_toggle:hover {
    background-color: #c0a080;
}

button#theme_toggle image {
    -gtk-icon-transform: none;
    color: #5c4b3a; /* Icon color for light mode */
    background-color: transparent; /* Ensure image background is transparent */
}

button#theme_toggle.dark {
    background-color: #000000;
    border: 1px solid #00ff00;
    border-radius: 50%; /* Keep circular shape in dark mode */
    min-width: 36px;
    min-height: 36px;
    padding: 8px;
    margin: 8px;
    box-shadow: 0 0 8px rgba(0, 255, 0, 0.5);
    overflow: hidden; /* Ensure content doesn't overflow the circular shape */
}

button#theme_toggle.dark:hover {
    background-color: #003300;
    box-shadow: 0 0 12px rgba(0, 255, 0, 0.8);
}

button#theme_toggle.dark image {
    -gtk-icon-transform: rotate(180deg); /* Rotate the icon in dark mode */
    color: #00ff00; /* Neon green icon for dark mode */
    background-color: transparent; /* Ensure image background is transparent */
}
";

//  TODO:
// - Create codeblocks on my website with markdown editor's output
// - rewrite in x86
// - live browser updates?
// - markdown + html syntax highlighting
//

impl MarkdownEditor {
    fn new(app: &Application) -> Self {

        let window = ApplicationWindow::new(app);
        window.set_title(Some("Markdown -> HTML *Attempt*"));
        window.set_default_size(800, 600);
        window.fullscreen(); // force fullscreen during demo recording

        // html template with proper indentation
        let html_template = String::from(
            "<!DOCTYPE html>\n\
             <html lang=\"en\">\n\
             <head>\n\
             \t<meta charset=\"UTF-8\">\n\
             \t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
             \t<meta name=\"description\" content=\"e-sayin website description\">\n\
             \t<meta name=\"color-scheme\" content=\"light-only\">\n\
             \t<link rel\"stylesheet\" href=\"../style.css\">\n\
             </head>\n\
             <body>\n\
             \n\
             {}\n\
             </body>\n\
             </html>"
        );

        // create split pane
        let paned = Paned::new(Orientation::Horizontal);
        window.set_child(Some(&paned));

        // left side
        let left_md_input = ScrolledWindow::new();
        let text_view = TextView::new();
        text_view.set_widget_name("markdown_input");
        text_view.set_wrap_mode(WrapMode::Word); // Wrap at word boundaries
        
        // Enable paste handling
        text_view.set_accepts_tab(true);
        
        let md_buffer = text_view.buffer();
        left_md_input.set_child(Some(&text_view));
        paned.set_start_child(Some(&left_md_input));
        paned.set_resize_start_child(true);
        paned.set_shrink_start_child(false);

        // right side
        let scrolled_html = ScrolledWindow::new();
        let html_view = TextView::new();
        html_view.set_widget_name("static_html");
        html_view.set_editable(false); // Make it read-only
        html_view.set_can_focus(false); // Prevent focus on HTML view
        html_view.set_wrap_mode(WrapMode::None); // Allow horizontal scrolling
        
        // Create an overlay to hold the ScrolledWindow and the theme toggle button
        let overlay = Overlay::new();
        
        // Create the theme toggle button
        let theme_button = Button::new();
        theme_button.set_widget_name("theme_toggle");
        theme_button.set_halign(Align::End);
        theme_button.set_valign(Align::End);
        theme_button.set_size_request(36, 36); // Set fixed size for the button
        
        // Add a moon icon for the button
        let icon = Image::from_icon_name("weather-clear-night-symbolic");
        theme_button.set_child(Some(&icon));
        
        // Use CSS to control styling
        let css_provider = CssProvider::new();
        css_provider.load_from_data(LIGHT_MODE_CSS);
        
        // Create a separate CSS provider for the button
        let button_css_provider = CssProvider::new();
        button_css_provider.load_from_data(THEME_BUTTON_CSS);
        
        // Apply CSS to both text views and the window
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window), 
            &css_provider, 
            STYLE_PROVIDER_PRIORITY_APPLICATION
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&html_view), 
            &css_provider, 
            STYLE_PROVIDER_PRIORITY_APPLICATION
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&text_view), 
            &css_provider, 
            STYLE_PROVIDER_PRIORITY_APPLICATION
        );
        
        // Apply CSS to the button
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&theme_button), 
            &button_css_provider, 
            STYLE_PROVIDER_PRIORITY_APPLICATION
        );
        
        // padding to make HTML view nicer
        html_view.set_left_margin(10);
        html_view.set_right_margin(10);
        html_view.set_top_margin(10);
        html_view.set_bottom_margin(10);
        
        // Add a click controller to the HTML view to redirect focus to markdown input
        let click_controller = gtk4::GestureClick::new();
        let text_view_clone = text_view.clone();
        click_controller.connect_pressed(move |_, _, _, _| {
            text_view_clone.grab_focus();
        });
        html_view.add_controller(click_controller);

        let html_buffer = html_view.buffer();
        scrolled_html.set_child(Some(&html_view));
        
        // Add the ScrolledWindow to the overlay
        overlay.set_child(Some(&scrolled_html));
        
        // Add the button as an overlay
        overlay.add_overlay(&theme_button);
        
        // Add the overlay to the paned view
        paned.set_end_child(Some(&overlay));
        paned.set_resize_end_child(true);
        paned.set_shrink_end_child(false);

        // initial empty HTML structure
        html_buffer.set_text(&html_template.replace("{}", ""));
        
        // Ensure markdown input has focus when app starts
        text_view.grab_focus();

        // create the markdown chars set!
        let mut markdown_chars = HashSet::new();
        for c in "#*_-+[]()>`".chars() {
            markdown_chars.insert(c);
        }

        // Create a reference to is_dark_mode for the button click handler
        // Randomly choose initial theme with 50% chance
        let mut rng = rand::thread_rng();
        let initial_dark_mode = rng.gen_bool(0.5);
        let is_dark_mode = Rc::new(RefCell::new(initial_dark_mode));
        
        // Apply initial theme based on random selection
        if initial_dark_mode {
            css_provider.load_from_data(DARK_MODE_CSS);
            theme_button.add_css_class("dark");
        } else {
            css_provider.load_from_data(LIGHT_MODE_CSS);
        }
        
        // Connect the button click signal
        let is_dark_mode_clone = is_dark_mode.clone();
        let css_provider_clone = css_provider.clone();
        let text_view_clone = text_view.clone();
        theme_button.connect_clicked(move |button| {
            let mut dark_mode = is_dark_mode_clone.borrow_mut();
            *dark_mode = !*dark_mode;
            
            if *dark_mode {
                css_provider_clone.load_from_data(DARK_MODE_CSS);
                button.add_css_class("dark");
            } else {
                css_provider_clone.load_from_data(LIGHT_MODE_CSS);
                button.remove_css_class("dark");
            }
            
            // Return focus to markdown input after theme change
            text_view_clone.grab_focus();
        });
        
        // Add a click controller to the theme button to prevent focus change
        let click_controller_button = gtk4::GestureClick::new();
        //let text_view_clone = text_view.clone();
        click_controller_button.connect_pressed(move |_, _, _, _| {
            // This will be called before the clicked signal
            // We'll let the clicked signal handle the theme change
            // and then it will restore focus
        });
        theme_button.add_controller(click_controller_button);
        
        MarkdownEditor {
            window,
            md_buffer,
            html_buffer,
            update_pending: Rc::new(RefCell::new(false)),
            html_template,
            markdown_chars,
            //is_dark_mode,
            //css_provider,
        }
    }

    fn connect_signals(&self) {
        // clone values for use in closures
        let md_buffer = self.md_buffer.clone();
        let html_buffer = self.html_buffer.clone();
        let html_template = self.html_template.clone();
        let update_pending = self.update_pending.clone();
        let markdown_chars = self.markdown_chars.clone();


        // connect to insert_text signal
        self.md_buffer.connect_insert_text(move |buffer, location, text| {
            // get the line text including newly inserted text
            let line = location.line();
            let line_start = buffer.iter_at_line(line).expect("Failed to get line start"); // rust
            let mut line_end = buffer.iter_at_line(line).expect("Failed to get line end"); // is
            line_end.forward_to_line_end();                                                // clean

            // get the complete line text after insertion
            let mut temp_end = location.clone();
            temp_end.forward_chars(text.len() as i32);
            let before_insert = buffer.text(&line_start, location, false);
            let after_insert = buffer.text(&temp_end, &line_end, false);
            let complete_line = format!("{}{}{}", before_insert, text, after_insert);

            // Check for proper list marker pattern
            let trimmed = complete_line.trim_start();
            let is_list_marker = trimmed.starts_with('-') || trimmed.starts_with('+') || trimmed.starts_with('*');

            // Update immediately for proper markdown syntax
            let has_markdown_char = text.chars().any(|c| markdown_chars.contains(&c));
            
            if is_list_marker || has_markdown_char {
                update_preview(&md_buffer, &html_buffer, &html_template);
            } else if !*update_pending.borrow() {
                *update_pending.borrow_mut() = true;
                
                let tb = md_buffer.clone();
                let hb = html_buffer.clone();
                let ht = html_template.clone();
                let up = update_pending.clone();
                
                timeout_add_local(Duration::from_millis(200), move || {
                    update_preview(&tb, &hb, &ht);
                    *up.borrow_mut() = false;
                    glib::Continue(false)
                });
            }
        });

        // Connect to delete_range signal
        let tb = self.md_buffer.clone();
        let hb = self.html_buffer.clone();
        let ht = self.html_template.clone();
        let up = self.update_pending.clone();
        
        self.md_buffer.connect_delete_range(move |_, _, _| {
            if !*up.borrow() {
                *up.borrow_mut() = true;
                
                let tb2 = tb.clone();
                let hb2 = hb.clone();
                let ht2 = ht.clone();
                let up2 = up.clone();
                
                timeout_add_local(Duration::from_millis(200), move || {
                    update_preview(&tb2, &hb2, &ht2);
                    *up2.borrow_mut() = false;
                    glib::Continue(false)
                });
            }
        });

        // Connect to the changed signal to catch all modifications including paste operations
        let tb = self.md_buffer.clone();
        let hb = self.html_buffer.clone();
        let ht = self.html_template.clone();
        let up = self.update_pending.clone();
        
        self.md_buffer.connect_changed(move |_| {
            if !*up.borrow() {
                *up.borrow_mut() = true;
                
                let tb2 = tb.clone();
                let hb2 = hb.clone();
                let ht2 = ht.clone();
                let up2 = up.clone();
                
                // Update immediately for paste operations
                update_preview(&tb2, &hb2, &ht2);
                
                // reset update pending after delay
                // but THERE IS NO delay Boi
                timeout_add_local(Duration::from_millis(20), move || {
                    *up2.borrow_mut() = false;
                    glib::Continue(false)
                });
            }
        });
    }

    fn show_all(&self) {
        self.window.show();
    }
}

fn update_preview(md_buffer: &TextBuffer, html_buffer: &TextBuffer, html_template: &str) {
    let start = md_buffer.start_iter();
    let end = md_buffer.end_iter();
    let markdown = md_buffer.text(&start, &end, false);

    // convert markdown and wrap in HTML structure
    let content = markdown_to_html(&markdown);
    
    // indent the content by 4 spaces to align with template
    let indented_content: String = content
        .lines()
        .map(|line| if !line.trim().is_empty() { format!("    {}", line) } else { line.to_string() })
        .collect::<Vec<String>>()
        .join("\n");
    
    let html = html_template.replace("{}", &indented_content);

    // update HTML buffer
    html_buffer.set_text(&html);
}

fn main() {
    // initialization
    let application = Application::builder()
        .application_id("x.LiL.quikMD")
        .build();

    application.connect_activate(|app| {
        let editor = MarkdownEditor::new(app);
        editor.connect_signals();
        editor.show_all();
    });

    

    application.run();
}
