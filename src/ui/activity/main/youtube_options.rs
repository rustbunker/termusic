/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use super::MainActivity;
use crate::ui::components::table;
use humantime::format_duration;
use std::time::Duration;
use tuirealm::props::{TableBuilder, TextSpan};
use tuirealm::PropsBuilder;
use unicode_truncate::{Alignment, UnicodeTruncateStr};

impl MainActivity {
    pub fn sync_youtube_options(&mut self, page_index: u32) {
        if self.youtube_options.is_empty() {
            if let Some(props) = self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
                if let Some(domain) = &self.invidious_instance.domain {
                    let props = table::TablePropsBuilder::from(props)
                        .with_table(
                            TableBuilder::default()
                                .add_col(TextSpan::from(format!(
                                    "Empty result.Probably {} is down.",
                                    domain
                                )))
                                .build(),
                        )
                        .build();
                    let msg = self
                        .view
                        .update(super::COMPONENT_SCROLLTABLE_YOUTUBE, props);
                    self.update(msg);
                }
            }

            return;
        }
        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = record.length_seconds;
            let duration_string = format!("{}", format_duration(Duration::from_secs(duration)));
            let duration_truncated = duration_string.unicode_pad(6, Alignment::Left, true);

            let title = record.title.as_str();

            table
                .add_col(TextSpan::new(
                    format!("[{}] ", duration_truncated,).as_str(),
                ))
                .add_col(TextSpan::new(title).bold());
        }
        let table = table.build();

        if let Some(props) = self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
            if let Some(domain) = &self.invidious_instance.domain {
                let title = format!(
                    "── Page {} ──┼─ {} ─┼─ {} ─────",
                    page_index, "Tab/Shift+Tab switch pages", domain,
                );
                let props = table::TablePropsBuilder::from(props)
                    .with_title(title, tuirealm::tui::layout::Alignment::Left)
                    .with_header(&["Duration", "Name"])
                    .with_widths(&[15, 85])
                    .with_table(table)
                    .build();
                self.view
                    .update(super::COMPONENT_SCROLLTABLE_YOUTUBE, props);
            }
        }
    }
}
