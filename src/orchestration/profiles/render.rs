//! HTML + chat renderers for profile artifacts.

use crate::orchestration::profiles::model::{ProfileMetadata, ProfileType};

/// Builds a minimal HTML document describing the profile.
pub fn build_profile_html(
    profile_type: ProfileType,
    metadata: &ProfileMetadata,
    highlights: &[String],
    fields: &[(String, String)],
) -> String {
    let title = format!("{} profile summary", profile_type_label(profile_type));
    let summary_list = if highlights.is_empty() {
        "<li>No highlights recorded yet.</li>".into()
    } else {
        highlights
            .iter()
            .map(|item| format!("<li>{}</li>", html_escape(item)))
            .collect::<Vec<String>>()
            .join("\n")
    };
    let field_rows = if fields.is_empty() {
        "<tr><td colspan=\"2\">No fields recorded.</td></tr>".into()
    } else {
        fields
            .iter()
            .map(|(label, value)| {
                format!(
                    "<tr><th>{}</th><td>{}</td></tr>",
                    html_escape(label),
                    html_escape(value)
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    };

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>{title}</title>
    <style>
      body {{ font-family: Arial, sans-serif; margin: 1.5rem; }}
      h1 {{ margin-bottom: 0; }}
      .meta {{ color: #555; margin-bottom: 1rem; }}
      ul {{ padding-left: 1.5rem; }}
      table {{ border-collapse: collapse; width: 100%; }}
      th, td {{ text-align: left; vertical-align: top; padding: 0.4rem; border-bottom: 1px solid #eee; }}
      th {{ width: 30%; color: #444; }}
    </style>
  </head>
  <body>
    <h1>{title}</h1>
    <div class="meta">Last updated: {updated}</div>
    <h2>Highlights</h2>
    <ul>
      {summary_list}
    </ul>
    <h2>Details</h2>
    <table>
      {field_rows}
    </table>
  </body>
</html>
"#,
        title = title,
        updated = metadata.last_updated.to_rfc3339(),
        summary_list = summary_list,
        field_rows = field_rows
    )
}

fn profile_type_label(profile_type: ProfileType) -> &'static str {
    match profile_type {
        ProfileType::User => "User",
        ProfileType::Work => "Work",
        ProfileType::Writing => "Writing",
        ProfileType::Knowledge => "Knowledge",
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
