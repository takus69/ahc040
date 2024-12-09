use proconio::input_interactive;
use std::collections::{VecDeque, BinaryHeap, HashSet};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

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

    fn add(&mut self, r#box: &mut Box, i: usize, r: usize, d: char, col_i: usize) {
        r#box.add(self.wh[i], r, d, col_i);

        // ローカル実行の場合は本当のサイズのBoxを算出
        if cfg!(debug_assertions) {
            self.judge.real_box.add(self.judge.wh[i], r, d, col_i);
        }
    }

    fn playout(&mut self, org_box: &Box, next_i: usize) -> usize {
        let mut r#box = org_box.clone();
        for i in next_i..self.n {
            let col_i = r#box.start_col_i;
            let (r, d) = r#box.opt_instruction(self.wh[i], 'L', col_i);
            self.add(&mut r#box, i, r, d, col_i);
        }

        r#box.get_score()
    }

    fn optimize(&mut self) -> BinaryHeap<(usize, Box)> {
        // 乱数
        let seed: u64 = 42;
        let mut rng = StdRng::seed_from_u64(seed);

        // 最大のhを決める
        let mut sum_area = 0;
        for (wi, hi) in self.wh.iter() {
            sum_area += wi * hi;
        }
        let ref_h = (sum_area as f64).sqrt() as usize;

        // ビーム
        let beams_width = self.t;
        let trial = 600 / self.n;
        let mut beams: BinaryHeap<(usize, Box)> = BinaryHeap::new();
        let r#box = Box::new(ref_h);
        let mut hashes: HashSet<u64> = HashSet::new();
        hashes.insert(calculate_hash(&r#box.order));
        beams.push((usize::MAX, r#box));

        // グッズを順に追加する
        for i in 0..self.n {
            self.judge.comment(format!("i: {}", i));
            let mut next_beams: BinaryHeap<(usize, Box)> = BinaryHeap::new();
            while !beams.is_empty() {
                let (_, mut r#box) = beams.pop().unwrap();
                let org_box = r#box.clone();
                for j in 0..trial {
                    // パラメータをランダム選択
                    let mut r = rng.gen_range(0..=1);
                    let mut d = 'L';
                    let mut col_i = rng.gen_range(r#box.start_col_i..=r#box.col_info.len());
                    if j == 0 {
                        (r, d) = r#box.opt_instruction(self.wh[i], 'L', r#box.start_col_i);
                        col_i = r#box.start_col_i;
                    }
                    // ハッシュ関数で同一状態を確認
                    let mut order = r#box.order.clone();
                    let b = if col_i == r#box.col_info.len() { -1 } else { r#box.col_info[col_i].2 as isize };
                    order.push((r, d, b));
                    let hash = calculate_hash(&order);
                    if hashes.contains(&hash) { continue; } else { hashes.insert(hash); }
                    self.add(&mut r#box, i, r, d, col_i);  // グッズを配置

                    // プレイアウト
                    let score = self.playout(&r#box, i+1);
                    self.judge.comment(format!("trial: {}, hash: {:X}, playout score: {}", j, hash, score));
                    next_beams.push((score, r#box));
                    while next_beams.len() > beams_width {
                        next_beams.pop();
                    }
                    r#box = org_box.clone();
                }
            }
            beams = next_beams;
            hashes = HashSet::new();
        }

        beams
    }

    fn solve(&mut self) {
        let mut opt_estimated_score = usize::MAX;
        let mut opt_score = usize::MAX;
        let mut opt_real_score = usize::MAX;

        let mut beams = self.optimize();
        // Vecに格納しなおして、スコアの小さい方から順に回答する
        let mut target: VecDeque<Box> = VecDeque::new();
        while !beams.is_empty() {
            let (_, r#box) = beams.pop().unwrap();
            target.push_back(r#box);
        }

        for _ in 0..self.t {
            if target.is_empty() {
                let r#box = Box::new(0);
                self.ans(r#box);
                continue;
            }
            let r#box = target.pop_back().unwrap();

            let (estimated_score, score, real_score) = self.ans(r#box.clone());

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
        let real_box = Box::new(0);
        
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
        self.real_box = Box::new(self.real_box.ref_h);
        (w, h)
    }

    fn comment(&self, text: String) {
        println!("# {}", text);
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Box {
    goods: Vec<(usize, usize)>,
    status: Vec<(usize, usize, usize, usize)>,
    max_x: usize,
    max_y: usize,
    col_info: Vec<(usize, usize, usize)>,  // (max_w, sum_h, last_i)
    area_info: Vec<usize>,
    order: Vec<(usize, char, isize)>,
    start_col_i: usize,
    ref_h: usize,
}

impl Ord for Box {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.max_x + self.max_y).cmp(&(other.max_x + other.max_y))
    }
}

impl PartialOrd for Box {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((self.max_x + self.max_y).cmp(&(other.max_x + other.max_y)))
    }
}

impl Box {
    fn new(ref_h: usize) -> Box {
        let goods = Vec::new();
        let status = Vec::new();
        let max_x = 0;
        let max_y = 0;
        let col_info = Vec::new();  // 列に追加したグッズの(max_w, sum_h, last_i)
        let area_info = Vec::new();  // 列に追加したグッズの面積の合計
        let order = Vec::new();
        let start_col_i = 0;
        Box { goods, status, max_x, max_y, col_info, area_info, order, start_col_i, ref_h }
    }

    fn opt_instruction(&self, wh: (usize, usize), d: char, col_i: usize) -> (usize, char) {
        let mut opt_r = 0;
        let mut opt_eval = 0.0;
        let i = self.goods.len();
        let b = if col_i == self.col_info.len() { -1 } else { self.col_info[col_i].2 as isize };

        // 回転の評価
        for r in [0, 1] {
            // 一つ前の位置を取得
            let (x1, y1, x2, y2) = self.put(wh, r, d, b);
            let (mut max_x, mut sum_y, mut last_i) = match self.col_info.last() {
                Some(&last) => { last },
                None => { (0, 0, i) },
            };
            max_x = max_x.max(x2);
            sum_y += y2-y1;
            let (pre_max_x, _, _) = if self.col_info.len() > 1 { self.col_info[self.col_info.len()-1] } else { (0, 0, i) };
            let area = match self.area_info.last() {
                Some(&last) => last,
                None => 0,
            };
            let eval = (area + (wh.0 * wh.1)) as f64 / (sum_y * (max_x - pre_max_x)) as f64;
            if eval > opt_eval {
                opt_eval = eval;
                opt_r = r;
            }
        }

        (opt_r, d)
    }

    fn add(&mut self, wh: (usize, usize), r: usize, d: char, col_i: usize) {
        let mut i = self.goods.len();
        let b = if col_i == self.col_info.len() { -1 } else { self.col_info[col_i].2 as isize };
        if b == -1 {
            self.col_info.push((0, 0, i));
            self.area_info.push(0);
        }

        self.goods.push(wh);
        self.order.push((r, d, b));

        let (x1, y1, x2, y2) = self.put(wh, r, d, b);

        self.status.push((x1, y1, x2, y2));
        self.max_x = self.max_x.max(x2);
        self.max_y = self.max_y.max(y2);
        let (mut max_x, mut sum_y, _) = self.col_info[col_i];
        let (pre_max_x, _, _) = if col_i > 0 { self.col_info[col_i-1] } else { (0, 0, 0) };
        if pre_max_x > x2 {
            self.col_info[col_i] = (max_x, sum_y, self.col_info[col_i].2);
        } else {
            max_x = max_x.max(x2);
            sum_y += y2-y1;
            self.col_info[col_i] = (max_x, sum_y, i);
            self.area_info[col_i] += wh.0 * wh.1;
        }
        if sum_y > self.ref_h {
            self.start_col_i = col_i + 1;
        }
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
