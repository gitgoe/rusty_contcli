use futures::future;
use futures::future::FutureExt;
use log::debug;
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use std::io::{Error, ErrorKind};
use rand::Rng;
use std::collections::HashMap;
use map_macro::hash_map;
use futures::{stream, StreamExt};


#[derive(Debug,PartialEq, Clone)]
struct Client {
    id: i32,
    contrats: Vec<String>,
}

impl Client {
    fn new( id: i32, contrats: Vec<String>) -> Self {
        Self { id: id, contrats: contrats }
    }    
}

#[derive(Debug,PartialEq)]
struct Contrat {
    id: String,
}

impl Contrat {
    fn new( id: &str) -> Self {
        Self { id: id.to_string()}
    }
}

#[derive(Debug, PartialEq)]
struct ContratClient {
    client: Client,
    contrats: HashMap<String, Contrat>,
}

struct DB{
    pub clients: Vec<Client>,
    pub contrats: Vec<Contrat>,
}

impl DB {
    fn init()-> Self{
        let clients = vec![
            Client::new(1, vec!["101".to_string(),"102".to_string(),"103".to_string()]),
            Client::new(2, vec!["201".to_string()]),
            Client::new(3, vec!["301".to_string(),"302".to_string()]),
            Client::new(4, vec!["401".to_string(),"402".to_string(),"403".to_string(),"404".to_string()]),
            Client::new(5, vec!["501".to_string(),"502".to_string(),"503".to_string(),"504".to_string(),"505".to_string(),"506".to_string(),"507".to_string(),"508".to_string(),"509".to_string()]),
           ];
           let contrats = vec![
            Contrat::new("101"),
            Contrat::new("102"),
            Contrat::new("103"),
            Contrat::new("201"),
            Contrat::new("301"),
            Contrat::new("302"),
            Contrat::new("401"),
            Contrat::new("402"),
            Contrat::new("403"),
            Contrat::new("404"),
            Contrat::new("501"),
            Contrat::new("502"),
            Contrat::new("503"),
            Contrat::new("504"),
            Contrat::new("505"),
            Contrat::new("506"),
            Contrat::new("507"),
            Contrat::new("508"),
            Contrat::new("509"),
            ];
       
        Self {clients: clients, contrats: contrats}
    }
}


struct FakeRequest <A,B> {
    request_body: A,
    response_body: B
}

struct FakeResponse <B>{body: B}
struct FakeHttpClient{}

impl FakeHttpClient {
    fn send <A,B>(&self, req: FakeRequest<A,B>) -> impl Future<Output = Result<FakeResponse<B>, Error>> {
        let delay = rand::thread_rng().gen_range(10..=500);
        sleep(Duration::from_millis(500)).then(move |_| {
            if delay < 10  {
                return  future::err(Error::new(ErrorKind::Interrupted, "server unavailable!!!"));
             }
             future::ok(FakeResponse{body: req.response_body})  
        })
        
    }
}

async fn fetch_client(id: i32) -> Result<Option<Client>,Error>{
    let db =  DB::init();    
    FakeHttpClient{}.send(FakeRequest { request_body: id, response_body: db.clients.into_iter().find(|x| x.id == id)}).await.map(|f| f.body)    
}

async fn fetch_contrat(id: String) -> Result<Option<Contrat>,Error>{
    let db =  DB::init();
    FakeHttpClient{}.send(FakeRequest { request_body: id.clone(), response_body: db.contrats.into_iter().find(|x| x.id == id)}).await.map(|f| f.body)    
}

async fn fetch_all_contrats_in_parallel(cli: Client) -> Vec<(i32,Contrat)> {
    let futures: Vec<_> = cli.contrats
        .into_iter()
        .inspect(|_|debug!("start new thread contrat={}",cli.id) )
        .map(|id| tokio::spawn(fetch_contrat(id)))
        .collect();
    
    let mut res = Vec::with_capacity(futures.len());
    for f in futures.into_iter() {
        let cont = f.await.unwrap().unwrap().unwrap();
        debug!("wait thread to finish for client={}",cli.id);
        res.push((cli.id, cont));
    }
  res
}

async fn fetch_all_contrats(cli: Client) ->  Vec<(i32,Contrat)> {
    let mut accumulator = Vec::<(i32,Contrat)>::new();    
    for id in cli.contrats.into_iter() {
        let contrat = fetch_contrat(id).await.unwrap();
        match contrat {
            Some(c) =>  accumulator.push((cli.id, c)),
            None    => println!("contrat not found"),
        };
      }
      accumulator
}

 fn fetch_clients_in_batch(closure: Vec<ContratClient>) {

    let expected = vec![
        ContratClient {
            client:  Client::new(1, vec!["101".to_string(),"102".to_string(),"103".to_string()]),
            contrats: hash_map! {
                "101".to_string() =>  Contrat::new("101"),
                "102".to_string() =>  Contrat::new("102"),
                "103".to_string() =>  Contrat::new("103"),
            }
        },
        ContratClient {
            client:  Client::new(3, vec!["301".to_string(),"302".to_string()]),
            contrats: hash_map! {
                "301".to_string() =>  Contrat::new("301"),
                "302".to_string() =>  Contrat::new("302"),
            }
        },
        ContratClient {
            client:  Client::new(5, vec!["501".to_string(),"502".to_string(), "503".to_string(), "504".to_string(),"505".to_string(), "506".to_string(), "507".to_string(), "508".to_string(), "509".to_string()]),
            contrats: hash_map! {
                "301".to_string() =>  Contrat::new("301"),
                "302".to_string() =>  Contrat::new("302"),
                "303".to_string() =>  Contrat::new("303"),
                "304".to_string() =>  Contrat::new("304"),
                "305".to_string() =>  Contrat::new("305"),
                "306".to_string() =>  Contrat::new("306"),
                "307".to_string() =>  Contrat::new("307"),
                "308".to_string() =>  Contrat::new("308"),
                "309".to_string() =>  Contrat::new("309"),
            }
        },

    ];
    
    let was = closure;

    debug!("was ={:?}", was);

    let diff: Vec<_> = was.into_iter().filter(|item| expected.contains(item)).collect();

    if diff.len() > 0 {
        debug!("difference ={}", diff.len());
    }
}



#[tokio::main(flavor = "multi_thread")]
async fn main() {

     // Initialize simplelog logging
    let config = ConfigBuilder::new()
    .set_target_level(LevelFilter::Trace)
    .build();
    let _ = SimpleLogger::init(LevelFilter::Debug, config); 

    let task =
        stream::iter(vec![1,5]).map(|id|  async move {  
        let client = fetch_client(id).await.unwrap().unwrap();   
        let contrats = fetch_all_contrats_in_parallel(client.clone()).await;
        let mapping: HashMap<String, Contrat> = contrats.into_iter().map(|c| (c.0.to_string(), c.1)).collect::<HashMap<_, _>>();
        ContratClient{client: client.clone(), contrats: mapping}
    }).buffer_unordered(5).collect::<Vec<_>>().await;
    

    fetch_clients_in_batch(task);

    debug!("end"); 

}


#[tokio::test]
async fn test_fetch_client() {
    let cli = fetch_client(1).await;
    debug!("cli={:?}",cli);  
    assert_eq!(cli.unwrap(), Some(Client { id: 1, contrats: vec!["101".to_string(), "102".to_string(), "103".to_string()] }));  
}

#[tokio::test]
async fn test_fetch_contrat() {
    let cont = fetch_contrat("101".to_string()).await;
    debug!("cont={:?}",cont);   
    assert_eq!(cont.unwrap(), Some(Contrat { id: "101".to_string() }));   
}

#[tokio::test]
async fn test_fetch_all_contrats() {
    let cont = fetch_all_contrats( Client::new(2, vec!["201".to_string()])).await;
    debug!("cont={:?}",cont);    
    assert_eq!(cont,vec![(2, Contrat { id: "201".to_string() })] );   
}
