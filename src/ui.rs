use crate::model::Model;

pub fn build_html_header() -> String {
    return String::from(r#"
    <html>
        <head>
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <meta charset="UTF-8">
        </head>
        <body>"#);
}

pub fn build_model_editor(model: &Model) -> String {
    return format!("\n\t\t<div>{}</div>",model.name);
}

pub fn build_html_footer() -> String {
    return String::from(r#"
        </body>
    </html>"#);
}