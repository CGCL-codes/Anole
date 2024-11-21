#[macro_use]
extern crate log;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chain_demo_simchain::SimChain;
use futures::StreamExt;
use serde::Serialize;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;
use chain_demo::*;

static mut CHAIN: Option<SimChain> = None;

fn get_chain() -> &'static SimChain {
    unsafe { CHAIN.as_ref().unwrap() }
}

#[derive(Debug)]
struct MyErr(anyhow::Error);

impl fmt::Display for MyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.0.to_string())
    }
}

fn handle_err<E: fmt::Display + fmt::Debug + Send + Sync + 'static>(e: E) -> MyErr {
    MyErr(anyhow::Error::msg(e))
}

impl actix_web::error::ResponseError for MyErr {}

macro_rules! impl_get_info {
    ($name: ident, $func: ident) => {
        async fn $name(req: web::Path<(IdType,)>) -> actix_web::Result<impl Responder> {
            let id = req.into_inner().0;
            info!("call {} with {}", stringify!($func), id);
            let data = get_chain().$func(id).map_err(handle_err)?;
            Ok(HttpResponse::Ok().json(data))
        }
    };
}

impl_get_info!(web_get_blk_header, read_block_header);
impl_get_info!(web_get_blk_data, read_block_data);
impl_get_info!(web_get_inter_index, read_inter_index);
impl_get_info!(web_get_transaction, read_transaction);



async fn web_get_param() -> actix_web::Result<impl Responder> {
    info!("call get_parameter");
    let data = get_chain().get_parameter().map_err(handle_err)?;
    Ok(HttpResponse::Ok().json(data))
}

async fn web_get_inter_indexs() -> actix_web::Result<impl Responder> {
    info!("call get_inter_indexs");
    let data = get_chain().read_inter_indexs().map_err(handle_err)?;
    Ok(HttpResponse::Ok().json(data))
}

async fn web_query(query_param: web::Json<QueryParam>) -> actix_web::Result<impl Responder>{
    info!("into web_query");
    let result = historical_query(&query_param, get_chain()).map_err(handle_err)?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pass: bool,
    fail_detail: VerifyResult,
    verify_time_in_ms: u64,
    vo_size:usize,
}

async fn web_verify(mut body: web::Payload) -> actix_web::Result<impl Responder>{
    // transfer from stream to OverallResult
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item?);
    }
    let query_res: OverallResult = serde_json::from_slice(&bytes).map_err(handle_err)?;
    let (verify_result, time) = query_res.verify(get_chain()).await.map_err(handle_err)?;
    let response = VerifyResponse{
        pass: verify_result.is_ok(),
        fail_detail: verify_result,
        verify_time_in_ms: time.as_millis() as u64,
        vo_size:query_res.vo_size
    };
    Ok(HttpResponse::Ok().json(response))
}


#[derive(StructOpt, Debug)]
#[structopt(name = "simchain-server")]
struct Opts {
    /// input db path
    #[structopt(short = "-db", long, parse(from_os_str))]
    db_path: PathBuf,

    /// server binding address
    #[structopt(short, long, default_value = "127.0.0.1:8000")]
    binding: String,
}

#[actix_rt::main]
async fn main() -> actix_web::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));
    let opts = Opts::from_args();
    let chain = SimChain::open(&opts.db_path).map_err(handle_err)?;
    unsafe {
        CHAIN = Some(chain);
    }

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST"]),
            )
            .route("/get/param", web::get().to(web_get_param))
            .route("/get/blk_header/{id}", web::get().to(web_get_blk_header))
            .route("/get/blk_data/{id}", web::get().to(web_get_blk_data))
            .route("/get/inter_index/{timestamp}", web::get().to(web_get_inter_index))
            .route("/get/inter_indexs", web::get().to(web_get_inter_indexs))
            .route("/get/tx/{id}", web::get().to(web_get_transaction))
            .route("/query", web::post().to(web_query))
            .route("/verify", web::post().to(web_verify))
    })
    .bind(opts.binding)?
    .run()
    .await?;

    Ok(())
}
