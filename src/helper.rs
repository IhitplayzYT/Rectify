pub mod Helper{
    use std::process::exit;



    const DBG_STR: &str = "";
    const OK:i32 = 0;
    const ERR:i32 = -1;


    #[derive(Debug,Clone)]
    pub struct CLI{
        pub dbg: bool,
        pub ipipe: Option<String>,
        pub opipe: Option<String>,
        pub sess: bool,
    }


    pub fn Help(){
        println!("{DBG_STR}");
        exit(OK);
    }


    impl CLI{
        pub fn new() -> Self{
            Self {dbg: false,ipipe:None,opipe:None,sess:true}
        }

        pub fn Parse_Args(&mut self){
            let args: Vec<String> = std::env::args().skip(1).collect();
           for i in &args{
                if i == "-d" || i == "--debug" || i == " --DEBUG" || i == "-D"{
                    self.dbg = true;
                } else if i == "-h" || i == "--help" || i == " --HELP" || i == "-H"{
                    Help();
                } else if i.starts_with("--ipipe=") || i.starts_with("-ip="){
                    self.ipipe = Some(i[i.find("=").unwrap()+1..].to_string());
                } else if i.starts_with("--opipe=") || i.starts_with("-op="){
                    self.opipe = Some(i[i.find("=").unwrap()+1..].to_string());
                } else if i == "--session" || i == "--sess" || i == "-s" {
                    self.sess = if self.sess {false}else{true};
                }else{
                    Help();
                }
           } 
           match (self.ipipe.is_none(),self.opipe.is_none()){
            (true,true) => {                
                self.ipipe = Some("/tmp/iopipe".to_string());
                self.opipe = self.ipipe.clone();
            },
            (true,false) => {
                self.opipe = self.ipipe.clone();
            },
            (false,true) => {
                self.ipipe = self.opipe.clone()
            },
            _ => {}
           }
        }

    }


    





}