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

    fn ans(&mut self, r#box: Box) -> (usize, usize) {
        let estimated_score = r#box.get_score();
        let (w, h) = self.judge.ans(r#box);
        self.judge.comment(format!("turn: {}", self.judge.turn));
        self.judge.comment(format!("estimated score: {}", estimated_score));
        let score = w + h;
        self.judge.comment(format!("result score: {}", score));

        (estimated_score, score)
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
            r#box.add(self.wh[i], 0, 'L', pre_b);
            sum_h += self.wh[i].1;
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
        for _ in 0..self.t {
            let r#box = self.optimize();

            let (estimated_score, score) = self.ans(r#box);

            opt_estimated_score = opt_estimated_score.min(estimated_score);
            opt_score = opt_score.min(score);
        }

        // 結果出力
        self.judge.comment(format!("estimated score: {}, score: {}", opt_estimated_score, opt_score));
        eprintln!("{{ \"N\": {}, \"T\": {}, \"S\": {}, \"estimated score\": {}, \"score\": {} }}", self.n, self.t, self.s, opt_estimated_score, opt_score);
    }
}

struct Judge {
    n: usize,
    t: usize,
    s: usize,
    wh: Vec<(usize, usize)>,
    dwh: Vec<(isize, isize)>,
    turn: usize,
}

impl Judge {
    fn new(n: usize, t: usize, s: usize) -> Judge {
        let turn = 0;
        
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

        Judge { n, t, s, wh, dwh, turn }
    }

    fn ans(&mut self, r#box: Box) -> (usize, usize) {
        r#box.ans();
        
        let (w, h) = if cfg!(debug_assertions) {
            let (w, h) = (r#box.max_x, r#box.max_y);  // TODO: 実際のwhでスコアを再計算
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
    order: Vec<(usize, char, isize)>,
}

impl Box {
    fn new() -> Box {
        let goods = Vec::new();
        let status = Vec::new();
        let max_x = 0;
        let max_y = 0;
        let order = Vec::new();
        Box { goods, status, max_x, max_y, order }
    }

    fn add(&mut self, wh: (usize, usize), r: usize, d: char, b: isize) {
        self.goods.push(wh);
        self.order.push((r, d, b));

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

        self.status.push((x1, y1, x2, y2));
        self.max_x = self.max_x.max(x2);
        self.max_y = self.max_y.max(y2);
    }

    fn max_x(&self, in_y1: usize, in_y2: usize) -> usize {
        let mut max_x = 0;
        for &(_, x2, y1, y2) in self.status.iter() {
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
