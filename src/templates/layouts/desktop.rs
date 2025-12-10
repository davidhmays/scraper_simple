use maud::{html, Markup, DOCTYPE};

pub fn desktop_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }

                // Example CSS — remove or replace as needed
                style {
                    r#"
                    body { font-family: sans-serif; margin: 40px; }
                    .btn { padding: 8px 14px; background: #eee; border-radius: 6px; border: 1px solid #aaa; }
                    .card { padding: 16px; border: 1px solid #ccc; border-radius: 8px; margin-top: 20px; }
                    .card-body { margin-top: 8px; }
                    "#
                }
            }
            body {
                (content)
            }
        }
    }
}
