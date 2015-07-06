use std::{fmt, cmp};
use chrono::offset::{fixed,utc,TimeZone};
use chrono::{Datelike,Timelike};
use git2;

pub struct Heatmap {
    array: [u32; 24*7]
}

impl Heatmap {
    pub fn new() -> Heatmap {
        Heatmap { array: [0u32; 24*7] }
    }

    pub fn append(&mut self, time: &git2::Time) {

        let timestamp = utc::UTC.timestamp(time.seconds(), 0)
            .with_timezone(&fixed::FixedOffset::east(time.offset_minutes() * 60));

        let day = timestamp.weekday().num_days_from_monday();
        let hour = timestamp.hour();

        self.array[(day * 24 + hour) as usize] += 1;
    }

}

impl fmt::Display for Heatmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut vec = self.array.to_vec();
        vec.sort_by(|a, b| b.cmp(a));
        let max = cmp::max(1, vec[0]);

        const ARTS: [char; 5] = ['.', '▪', '◾', '◼', '⬛'];
        const DAYS: [&'static str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        try!(write!(f, "   "));
        for i in 0..24 {
            try!(write!(f, "{:3}", i));
        }
        try!(write!(f, "\n"));

        for day in 0..7 {
            try!(write!(f, "{}: ", DAYS[day]));
            for hour in 0..24 {
                try!(write!(f, "{: >3}", ARTS[(self.array[day * 24 + hour] as f32 / max as f32 * (ARTS.len() - 1) as f32) as usize]));
            }
            try!(write!(f, "\n"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use git2;
    use heatmap::Heatmap;

    #[test]
    fn smoke() {
        let mut hm = Heatmap::new();
        // Sun, 28 Jun 2015 13:17:20 +0600
        hm.append(&git2::Time::new(1435475840, 6*60));
        assert_eq!(hm.array[6 * 24 + 13], 1);
    }
}
