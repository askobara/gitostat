use std::{fmt,cmp};
use chrono::*;
use git2;

pub struct Heatmap {
    array: [u32; 24*7]
}

impl Heatmap {
    pub fn new() -> Heatmap {
        Heatmap { array: [0u32; 24*7] }
    }

    pub fn append(&mut self, time: &git2::Time) {

        let timestamp = UTC.timestamp(time.seconds(), 0)
            .with_timezone(&FixedOffset::east(time.offset_minutes() * 60));
        let (weekday, hour) = (timestamp.weekday().num_days_from_monday(), timestamp.hour());

        self.array[(weekday * 24 + hour) as usize] += 1;
    }
}

impl fmt::Display for Heatmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut vec = self.array.to_vec();
        vec.sort_by(|a, b| b.cmp(a));
        let max = vec[0];

        const arts: [char; 5] = ['.', '▪', '◾', '◼', '⬛'];
        const days: [&'static str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        write!(f, "   ");
        for i in 0..24 {
            write!(f, "{:3}", i);
        }
        write!(f, "\n");

        for i in 0..24*7 {
            if i % 24 == 0 {
                write!(f, "{}: ", days[i / 24]);
            }

            write!(f, "{:3}", arts[(self.array[i] as f32 / max as f32 * (arts.len() - 1) as f32) as usize]);

            if (i + 1) % 24 == 0 {
                write!(f, "\n");
            }
        }

        Ok(())
    }
}

