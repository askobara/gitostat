use std::{fmt,cmp};
use chrono::*;
use git2;

pub struct Heatmap {
    vec: [u32; 24*7]
}

impl Heatmap {
    pub fn new() -> Heatmap {
        Heatmap { vec: [0u32; 24*7] }
    }

    pub fn append(&mut self, time: &git2::Time) {

        let timestamp = UTC.timestamp(time.seconds(), 0)
            .with_timezone(&FixedOffset::east(time.offset_minutes() * 60));
        let (weekday, hour) = (timestamp.weekday().num_days_from_monday(), timestamp.hour());

        self.vec[(weekday * 24 + hour) as usize] += 1;
    }
}

impl fmt::Display for Heatmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // find max
        let mut max: u32 = 0;
        // TODO: sort this
        for count in self.vec.iter() {
            max = cmp::max(*count, max);
        }

        let arts = ['.', '▪', '◾', '◼', '⬛'];

        write!(f, " ");
        for i in 0..24 {
            write!(f, "{:3}", i);
        }
        write!(f, "\n");

        for i in 0..24*7 {
            if i % 24 == 0 {
                write!(f, "{}: ", i / 24);
            }
            write!(f, "{:3}", arts[(self.vec[i] as f32 / max as f32 * (arts.len() - 1) as f32) as usize]);
            // print!("{:3}", heatmap[i]);
            if (i + 1) % 24 == 0 {
                write!(f, "\n");
            }
        }

        Ok(())
    }
}

