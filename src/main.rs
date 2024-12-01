use proconio::{input, input_interactive};

struct Solver {
    n: usize,
    t: usize,
    s: usize,
    wh: Vec<(usize, usize)>,
    judge: Judge,
}

impl Solver {
    fn new(n: usize, t: usize, s: usize, wh: Vec<(usize, usize)>) -> Solver {
        let judge = Judge::new(n, t, s);
        Solver { n, t, s, wh, judge }
    }

    fn ans(&mut self, r#box: Box) -> (usize, usize, usize) {
        let estimated_score = r#box.get_score();
        let real_score = self.judge.real_box.get_score();
        self.judge.comment(format!("turn: {}", self.judge.turn+1));
        self.judge.comment(format!("estimated (w, h, score): ({}, {}, {})", r#box.max_x, r#box.max_y, estimated_score));
        self.judge.comment(format!("real (w, h, score): ({}, {}, {})", self.judge.real_box.max_x, self.judge.real_box.max_y, real_score));
        let (w, h) = self.judge.ans(r#box);
        let score = w + h;

        (estimated_score, score, real_score)
    }

    fn add(&mut self, r#box: &mut Box, i: usize, r: usize, d: char, b: isize) {
        r#box.add(self.wh[i], r, d, b);

        // ローカル実行の場合は本当のサイズのBoxを算出
        if cfg!(debug_assertions) {
            self.judge.real_box.add(self.judge.wh[i], r, d, b);
        }
    }

    fn optimize(&mut self) -> Box {
        let mut r#box = Box::new();
        let mut sum_area = 0;
        for (wi, hi) in self.wh.iter() {
            sum_area += wi * hi;
        }
        let ref_h = (sum_area as f64).sqrt() as usize;
        let mut pre_b = -1;
        let mut sum_h = 0;
        // グッズを順に追加する
        for i in 0..self.n {
            // 一つ前の下に合わせてref_hを超えたら次の行に行く
            let (r, d) = r#box.opt_instruction(self.wh[i], 'L', pre_b);
            self.add(&mut r#box, i, r, d, pre_b);
            sum_h += if r == 0 { self.wh[i].1 } else { self.wh[i].0 };
            pre_b = i as isize;
            if sum_h > ref_h {
                sum_h = 0;
                pre_b = -1;
            }
        }

        r#box
    }

    fn solve(&mut self) {
        let mut opt_estimated_score = usize::MAX;
        let mut opt_score = usize::MAX;
        let mut opt_real_score = usize::MAX;
        for _ in 0..self.t {
            let r#box = self.optimize();

            let (estimated_score, score, real_score) = self.ans(r#box);

            opt_estimated_score = opt_estimated_score.min(estimated_score);
            opt_score = opt_score.min(score);
            opt_real_score = opt_real_score.min(real_score);
        }

        // 結果出力
        self.judge.comment(format!("estimated score: {}, score: {}, real_score: {}", opt_estimated_score, opt_score, opt_real_score));
        eprintln!("{{ \"N\": {}, \"T\": {}, \"S\": {}, \"estimated score\": {}, \"score\": {}, \"real score\": {} }}", self.n, self.t, self.s, opt_estimated_score, opt_score, opt_real_score);
    }
}

struct Judge {
    n: usize,
    t: usize,
    s: usize,
    wh: Vec<(usize, usize)>,
    dwh: Vec<(isize, isize)>,
    turn: usize,
    real_box: Box,
}

impl Judge {
    fn new(n: usize, t: usize, s: usize) -> Judge {
        let turn = 0;
        let real_box = Box::new();
        
        // ローカル実行時
        let (wh, dwh) = if cfg!(debug_assertions) {
            input_interactive! {
                wh: [(usize, usize); n],
                dwh: [(isize, isize); t],
            }
            (wh, dwh)
        } else {
            (Vec::new(), Vec::new())
        };

        Judge { n, t, s, wh, dwh, turn, real_box }
    }

    fn ans(&mut self, r#box: Box) -> (usize, usize) {
        r#box.ans();
        
        let (w, h) = if cfg!(debug_assertions) {
            let (w, h) = (self.real_box.max_x, self.real_box.max_y);
            let (dw, dh) = self.dwh[self.turn];
            let (w, h) = (w as isize +dw, h as isize +dh);
            let w = if w > 0 { w as usize } else { 1 };
            let h = if h > 0 { h as usize } else { 1 };
            (w, h)
        } else {
            input_interactive! {
                w: usize,
                h: usize,
            }
            (w, h)
        };

        self.turn += 1;
        self.comment(format!("turn: {}", self.turn));
        self.comment(format!("estimated (w, h, score): ({}, {}, {})", r#box.max_x, r#box.max_y, r#box.get_score()));
        self.comment(format!("judge (w, h, score): ({}, {}, {})", w, h, w+h));
        self.comment(format!("real (w, h, score): ({}, {}, {})", self.real_box.max_x, self.real_box.max_y, self.real_box.get_score()));
        self.real_box = Box::new();
        (w, h)
    }

    fn comment(&self, text: String) {
        println!("# {}", text);
    }
}

struct Box {
    goods: Vec<(usize, usize)>,
    status: Vec<(usize, usize, usize, usize)>,
    max_x: usize,
    max_y: usize,
    sum_area: usize,
    order: Vec<(usize, char, isize)>,
}

impl Box {
    fn new() -> Box {
        let goods = Vec::new();
        let status = Vec::new();
        let max_x = 0;
        let max_y = 0;
        let sum_area = 0;
        let order = Vec::new();
        Box { goods, status, max_x, max_y, sum_area, order }
    }

    fn opt_instruction(&self, wh: (usize, usize), d: char, b: isize) -> (usize, char) {
        let mut opt_r = 0;
        let mut opt_eval = 0.0;

        for r in [0, 1] {
            let (x1, y1, x2, y2) = self.put(wh, r, d, b);
            let max_x = self.max_x.max(x2);
            let max_y = self.max_y.max(y2);
            let eval = (self.sum_area + (wh.0 * wh.1)) as f64 / (max_x * max_y) as f64;
            if eval > opt_eval {
                opt_eval = eval;
                opt_r = r;
            }
        }

        (opt_r, d)
    }

    fn add(&mut self, wh: (usize, usize), r: usize, d: char, b: isize) {
        self.goods.push(wh);
        self.order.push((r, d, b));

        let (x1, y1, x2, y2) = self.put(wh, r, d, b);

        self.status.push((x1, y1, x2, y2));
        self.max_x = self.max_x.max(x2);
        self.max_y = self.max_y.max(y2);
        self.sum_area += wh.0 * wh.1;
    }

    fn put(&self, wh: (usize, usize), r: usize, d: char, b: isize) -> (usize, usize, usize, usize) {
        let (dx, dy) = if r == 0 { wh } else { (wh.1, wh.0) };
        let (x1, y1, x2, y2) = if d == 'U' {
            // d == 'U'の場合
            let x1= if b == -1 { 0 } else { self.status[b as usize].2 };
            let x2 = x1 + dx;
            let y1 = self.max_y(x1, x2);
            let y2 = y1 + dy;
            (x1, y1, x2, y2)
        } else {
            // d == 'L'の場合
            let y1 = if b == -1 { 0 } else { self.status[b as usize].3 };
            let y2 = y1 + dy;
            let x1 = self.max_x(y1, y2);
            let x2 = x1 + dx;
            (x1, y1, x2, y2)
        };

        (x1, y1, x2, y2)
    }

    fn max_x(&self, in_y1: usize, in_y2: usize) -> usize {
        let mut max_x = 0;
        for &(_, y1, x2, y2) in self.status.iter() {
            if in_y1 < y2 && y1 < in_y2 {
                max_x = max_x.max(x2);
            }
        }

        max_x
    }

    fn max_y(&self, in_x1: usize, in_x2: usize) -> usize {
        let mut max_y = 0;
        for &(x1, _, x2, y2) in self.status.iter() {
            if in_x1 < x2 && x1 < in_x2 {
                max_y = max_y.max(y2);
            }
        }

        max_y
    }

    fn get_score(&self) -> usize {
        self.max_x + self.max_y
    }

    fn ans(&self) {
        let m = self.order.len();
        println!("{}", m);
        for i in 0..m {
            let (r, d, b) = self.order[i];
            println!("{} {} {} {}", i, r, d, b);
        }
    }
}

fn main() {
    input_interactive! {
        n: usize,
        t: usize,
        s: usize,
        wh: [(usize, usize); n],
    }

    let mut solver = Solver::new(n, t, s, wh);
    solver.solve();
}
