use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;
use crate::helpers::*;

extern crate rand;


fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    // orders.skip().perform_cmd( fetch_all() );
    orders.send_msg(Msg::Reset);
    Model { 
        button_disabled : false,
        source : get_source(&url),
        secret : get_secret(&url),
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        runs_queue : data::RunsQueue::empty(),
        // current_run: 0,
        center: None,
        // lock_frozen : true,
    }
}

struct Model {
    button_disabled : bool,
    source : Option<String>,
    secret : String,
    contest : data::ContestFile,
    runs: data::RunsFile,
    runs_queue : data::RunsQueue,
    // current_run: usize,
    center : Option<String>,
    // lock_frozen : bool,
}

enum Msg {
    Prox(usize),
    Scroll(usize),
    Prox1,
    Scroll1,
    // Wait,
    // Recalculate,
    // ToggleFrozen,
    // FetchedRuns(fetch::Result<data::RunsFile>),
    // FetchedContest(fetch::Result<data::ContestFile>),
    Reset,
    Fetched(
        fetch::Result<data::RunsFile>,
        fetch::Result<data::ContestFile>),
}

async fn fetch_all(source :Option<String>, secret : String) -> Msg {
    let r = fetch_allruns_secret(&source, &secret).await;
    let c = fetch_contest(&source).await;
    Msg::Fetched(r, c)
}

fn apply_all_runs_before_frozen(model: &mut Model) {

    for run in model.runs.sorted() {
        if run.time < model.contest.score_freeze_time {
            model.contest.apply_run(run).unwrap();
        }
        else {
            model.contest.apply_run_frozen(run).unwrap();

        }
    }
    model.runs_queue.setup_teams(&model.contest);
    model.contest.recalculate_placement().unwrap();
}

fn apply_one_run_from_queue(runs_queue: &mut data::RunsQueue, contest  : &mut data::ContestFile) {

    let _ = runs_queue.pop_run(contest);
    
}


fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Prox1 => {
            model.button_disabled = true;
            let next_center = model.runs_queue.queue.peek().map(|s| s.team_login.clone() );            
            if next_center == model.center {
                orders.send_msg(Msg::Scroll1);
            }
            else {
                let delay = match (&model.center, &next_center) {
                    (Some(c1), Some(c2)) => {
                        let p1 = model.contest.placement(c1).unwrap() as i64;
                        let p2 = model.contest.placement(c2).unwrap() as i64;

                        if (p1 - p2).abs() < 5 {
                            1000
                        }
                        else {
                            5000
                        }                       
                    },
                    _ => 5000,
                };
                // let delay = 1000;
                model.center = next_center;

                // let delay = 5000;
                orders.perform_cmd(cmds::timeout(delay, move || Msg::Scroll1));
            }
        },
        Msg::Scroll1 => {
            apply_one_run_from_queue(&mut model.runs_queue, &mut model.contest);

            model.contest.recalculate_placement().unwrap();
            model.button_disabled = false;

        },
        Msg::Prox(n) => {
            model.button_disabled = true;
            model.center = model.runs_queue.queue.peek().map(|s| s.team_login.clone() );
            orders.perform_cmd(cmds::timeout(5000, move || Msg::Scroll(n)));
        },
        Msg::Scroll(n) => {
            model.center = None;

            while model.runs_queue.queue.len() > n {
                apply_one_run_from_queue(&mut model.runs_queue, &mut model.contest);
            }
            model.contest.recalculate_placement().unwrap();
            model.button_disabled = false;

        },
        Msg::Fetched(Ok(runs), Ok(contest)) => {
            // model.current_run = 0;
            model.center = None;
            model.runs = runs;
            model.contest = contest;
            apply_all_runs_before_frozen(model);
            model.contest.reload_score().unwrap();
            // log!("run queue: ", model.runs_queue);
            model.button_disabled = false;
        },
        Msg::Fetched(Err(e), _) => {
            log!("fetched runs error!", e)
        },
        Msg::Fetched(_, Err(e)) => {
            log!("fetched contest error!", e)
        },
        Msg::Reset => {
            model.button_disabled = true;
            orders.skip().perform_cmd( fetch_all(model.source.clone(), model.secret.clone()) );
        }
    }
}

fn view(model: &Model) -> Node<Msg> {

    let button_disabled = if model.button_disabled { attrs!{At::Disabled => true} } else { attrs!{} };
    // let frozen = if model.lock_frozen {"Frozen Locked"} else { "Frozen Unlocked"};
    div![
        div![
            C!["commandpanel"],
            button!["+1", ev(Ev::Click, |_| Msg::Prox1),button_disabled.clone()],
            button!["Top 10", ev(Ev::Click, |_| Msg::Prox(10)),button_disabled.clone()],
            button!["Top 30", ev(Ev::Click, |_| Msg::Prox(30)),button_disabled.clone()],
            button!["Top 50", ev(Ev::Click, |_| Msg::Prox(50)),button_disabled.clone()],
            button!["Top 100", ev(Ev::Click, |_| Msg::Prox(100)),button_disabled.clone()],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset),button_disabled],
            // button![frozen, ev(Ev::Click, |_| Msg::ToggleFrozen),],
            div!["Times com runs pendentes: ", model.runs_queue.len()],
        ],
        div![
            style!{St::Position => "relative", St::Top => px(60)},
            views::view_scoreboard(&model.contest, &model.center, &None),
        ]
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
