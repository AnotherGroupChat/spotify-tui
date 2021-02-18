use super::util;
use crate::app::App;
use crate::error::VisualizationError;
use crate::user_config::VisualStyle;
use rhai::{
  serde::{from_dynamic, to_dynamic},
  Array, Engine, Scope, AST,
};
use tui::{
  backend::Backend,
  layout::{Constraint, Direction, Layout},
  style::Style,
  text::{Span, Spans},
  widgets::{BarChart, Block, Borders, Paragraph},
  Frame,
};

// Trait to produce widget
#[derive(Debug, Clone, serde::Deserialize)]
struct BarChartInfo {
  error: bool,
  labels: Vec<String>,
  counts: Vec<u64>,
}

pub fn draw<B>(f: &mut Frame<B>, app: &App)
where
  B: Backend,
{
  let margin = util::get_main_layout_margin(app);

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(5), Constraint::Length(75)].as_ref())
    .margin(margin)
    .split(f.size());

  let analysis_block = Block::default()
    .title(Span::styled(
      "Analysis",
      Style::default().fg(app.user_config.theme.inactive),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(app.user_config.theme.inactive));

  let visual_app = app.user_config.get_visualizer_or_default();
  let white = Style::default().fg(app.user_config.theme.text);
  let gray = Style::default().fg(app.user_config.theme.inactive);
  let tick_rate = app.user_config.behavior.tick_rate_milliseconds;
  let chart_title = &format!(
    "{} | Tick Rate {} {}FPS",
    visual_app.name,
    tick_rate,
    1000 / tick_rate,
  );
  let chart_block = Block::default()
    .borders(Borders::ALL)
    .style(white)
    .title(Span::styled(chart_title, gray))
    .border_style(gray);

  let empty_analysis_block = || {
    Paragraph::new("No analysis available")
      .block(analysis_block.clone())
      .style(Style::default().fg(app.user_config.theme.text))
  };
  let empty_chart_block = || {
    Paragraph::new("No information available")
      .block(chart_block.clone())
      .style(Style::default().fg(app.user_config.theme.text))
  };

  if let Some(analysis) = &app.audio_analysis {
    let progress_seconds = (app.song_progress_ms as f32) / 1000.0;

    let info: Vec<String> = &app.visualizer.analyze(analysis, progress_seconds);
    let texts: Vec<Spans> = info.iter().map(|span| Spans::from(span.clone())).collect();
    let p = Paragraph::new(texts)
      .block(analysis_block)
      .style(Style::default().fg(app.user_config.theme.text));
    f.render_widget(p, chunks[0]);

    let mut display_error: Option<VisualizationError> = match visual_app.style {
      VisualStyle::Bar => {
        // visualizer.compute_barchart(analysis, progress_seconds)
        let data: Result<Vec<(String, u64)>, VisualizationError> = {};
        match data {
          Ok(data) => {
            let data: Vec<(&str, u64)> =
              data.iter().map(|item| (item.0.as_str(), item.1)).collect();
            let width = (chunks[1].width) as f32 / (1 + data.len()) as f32;

            let analysis_bar = BarChart::default()
              .block(chart_block)
              .data(data.as_slice())
              .bar_width(width as u16)
              .bar_style(Style::default().fg(app.user_config.theme.analysis_bar))
              .value_style(
                Style::default()
                  .fg(app.user_config.theme.analysis_bar_text)
                  .bg(app.user_config.theme.analysis_bar),
              );
            f.render_widget(analysis_bar, chunks[1]);
            None
          }
          Err(err) => err,
        }
      }
      _ => VisualizationError::from("Unsupported type."),
    };
    if let Some(VisualizationError::Warning(message)) = display_error {
      let ts: Vec<Spans> = vec![Spans::from(message)];
      let p = Paragraph::new(ts)
        .block(chart_block)
        .style(Style::default().fg(app.user_config.theme.text));
      f.render_widget(p, chunks[1]);
    }
  } else {
    f.render_widget(empty_analysis_block(), chunks[0]);
    f.render_widget(empty_chart_block(), chunks[1]);
  }
}
