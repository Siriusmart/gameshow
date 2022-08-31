use std::{env, error::Error, collections::HashMap, fs::{self, OpenOptions}, io::{Write, self}};

use rand::{thread_rng, Rng};

const HELP_MSG: &'static str = r#"
Usage:
    With yourself: ./path_to_executable [file path] [leaderboard file] [rounds] [points per round] [penalty if wrong]
    With 1 player: ./path_to_executable [file path] [leaderboard file] [rounds] [points per round] [penalty if wrong] [player]
    With multiple players: ./path_to_executable [file path] [leaderboard file] [rounds] [points per round] [penalty if wrong] [players]
"#;

#[derive(Debug)]
struct Question {
    pub question: String,
    pub answer: String,
}

impl Question {
    pub fn to_string(&self, asked: bool) -> String {
        format!("{}{}|{}", if asked {
            "x_"
        } else {
            ""
        }, self.question, self.answer)
    }
}

struct LeaderBoard{
    pub map: HashMap<String, i32>,
    pub path: String,
}

impl LeaderBoard {
    pub fn load(path: String) -> Result<Self, Box<dyn Error>> {
        let file_content = fs::read_to_string(&path)?;
        let players = file_content.split("\n");

        let mut map: HashMap<String, i32> = HashMap::default();
        players.for_each(|player| {
            if player.len() == 0 {
                return;
            }
            let mut name_and_score = player.split(" ");
            
            let name = name_and_score.next().unwrap();
            let score = name_and_score.next().unwrap().parse().unwrap();

            map.insert(name.to_string(), score);
        });

        Ok(Self {
            map,
            path,
        })
    }

    pub fn update(&self) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&self.path)?;

        file.write(self.to_string().as_bytes())?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct LeaderBoardPlayer {
    pub score: i32,
    pub name: String,
}

impl ToString for LeaderBoard {
    fn to_string(&self) -> String {
        let mut out = Vec::new();
        for (name, score) in self.map.iter() {
            out.push(LeaderBoardPlayer{
                name: name.to_string(),
                score: *score,
            });
        }

        out.sort();

        out.into_iter().rev().map(|player| {
            player.to_string()
        }).collect::<Vec<_>>().join("\n")
    }
}

impl ToString for LeaderBoardPlayer {
    fn to_string(&self) -> String {
        format!("{} {}", self.name, self.score)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut args = env::args().skip(1);
    
    let file_path = match args.next() {
        None => {
            println!("{HELP_MSG}");
            return Ok(());
        },
        Some(content) if content == String::from("help") => {
            println!("{HELP_MSG}");
            return Ok(());
        },
        Some(content) => content,
    };
    let leaderboard_path = args.next().unwrap();
    let rounds: u32 = args.next().unwrap().parse()?;
    let points_per_round: i32 = args.next().unwrap().parse()?;
    let wrong_penalty: i32 = args.next().unwrap().parse()?;
    let mut leaderboard = LeaderBoard::load(leaderboard_path)?;
    let players = args.collect::<Vec<_>>();

    if players.len() == 0 && !leaderboard.map.contains_key("You") {
        leaderboard.map.insert(String::from("You"), 0);
    }

    for player in players.iter() {
        if !leaderboard.map.contains_key(player) {
            leaderboard.map.insert(player.clone(), 0);
        }
    }

    leaderboard.update()?;

    let mut not_asked = Vec::new();
    let mut asked = Vec::new();

    let file_content = fs::read_to_string(&file_path)?;
    file_content.split("\n").for_each(|mut question_and_answer| {
        if question_and_answer.len() == 0 {
            return;
        }

        let to_push = if question_and_answer.starts_with("x_") {
            question_and_answer = &question_and_answer[2..question_and_answer.len()];
            &mut asked
        } else {
            &mut not_asked
        };

        let question_and_answer: Vec<_> = question_and_answer.split("|").collect();
        let question = question_and_answer[0];

        to_push.push(Question{
            question: question.to_string(),
            answer: question_and_answer[1].to_string(),
        })
    });

    'main_loop: for round_number in 1..rounds + 1 {
        print!("Press enter to question");
        stdout.flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        drop(input);
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        stdout.flush()?;
        if not_asked.len() == 0 {
            println!("Seems like you have ran out of questions");
            return Ok(());
        }

        let question_index: usize = thread_rng().gen_range(0..not_asked.len());
        let question = not_asked.remove(question_index);

        print!("Round {}\nQuestion: {}\n\nPress enter once answered", round_number, question.question);
        stdout.flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        drop(input);

        print!("Answer: {}\n\nDid the player got it correct? [Y/n, s to skip]", question.answer);
        stdout.flush()?;

        let correct = loop {
            let mut input = String::new();
            stdin.read_line(&mut input)?;

            match input.trim() {
                "" | "y" => {
                    break true;
                },
                "n" => {
                    break false;
                },
                "s" => {
                    continue 'main_loop;
                }
                _ => {
                    print!("Invalid response, is the answer correct? [Y/n]");
                    stdout.flush()?;
                }
            }
        };

        let player = if players.len() == 0 {
            String::from("You")
        } else if players.len() == 1 {
            players[0].clone()
        } else {
            print!("Options:\n{}\n\nWho answered the question: ", players.iter().enumerate().map(|(index, player)| {
                format!("  [{}] {}", index + 1, player)
            }).collect::<Vec<_>>().join("\n"));
            stdout.flush()?;
            loop {
                let mut input = String::new();
                stdin.read_line(&mut input)?;

                match input.trim().parse::<usize>() {
                    Ok(index) => {
                        if index < players.len() {
                            break players[index - 1].clone();
                        }
                    },
                    Err(_) => {
                        stdout.flush()?;
                    }
                }

                print!("Invalid selection, please re-enter: ");
            }
        };

        *leaderboard.map.get_mut(&player).unwrap() += if correct {
            points_per_round
        } else {
            wrong_penalty
        };

        asked.push(question);
        update(&not_asked, &asked, &file_path)?;
        leaderboard.update()?;

    }

    update(&not_asked, &asked, &file_path)?;
    leaderboard.update()?;

    Ok(())
}

fn update(not_asked: &Vec<Question>, asked: &Vec<Question>, path: &str) -> Result<(), Box<dyn Error>> {
    let not_asked = not_asked.into_iter().map(|question| {
        question.to_string(false)
    }).collect::<Vec<_>>().join("\n");
    
    let asked = asked.into_iter().map(|question| {
        question.to_string(true)
    }).collect::<Vec<_>>().join("\n");

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .read(true)
        .open(path)?;

    file.write_all(format!("{not_asked}\n{asked}").as_bytes())?;

    Ok(())
}
