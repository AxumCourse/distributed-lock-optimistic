use std::sync::Arc;

/// 库存
#[derive(Debug, sqlx::FromRow)]
pub struct Inventory {
    pub id: i32,
    pub stock: i32,
    pub ver: i32,
}

/// 用于操作的数据ID
const ID: i32 = 1;
/// 用于模拟的并发数
const SPAWN_NUMS: usize = 10;

#[tokio::main]
async fn main() {
    let pool = get_pool().await.unwrap();
    let pool = Arc::new(pool);

    // -- 插入数据 --
    // let i = Inventory {
    //     id: ID,
    //     stock: (SPAWN_NUMS / 2) as i32,
    //     ver: 0,
    // };

    // create(&pool, &i).await.unwrap();
    // println!("插入成功");
    // return;

    // --- 模拟销售 ---
    let mut hs = vec![];
    for i in 0..SPAWN_NUMS {
        let pool = pool.clone();
        hs.push(tokio::spawn(sell(pool, ID as i32, i)));
    }
    for h in hs {
        let _ = h.await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // -- 查看详情 --
    let i = find(&pool, ID as i32).await.unwrap();
    println!("{:?}", i);
}

/// 获取数据库连接池
async fn get_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let dsn = std::env::var("PG_DSN")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1/draft".to_string());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&dsn)
        .await
}

async fn create(conn: &sqlx::PgPool, i: &Inventory) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO inventory (id, stock, ver) VALUES ($1, $2, $3)")
        .bind(i.id)
        .bind(i.stock)
        .bind(i.ver)
        .execute(conn)
        .await?;
    Ok(())
}

async fn sell(conn: Arc<sqlx::PgPool>, id: i32, i: usize) {
    let mut tx = conn.begin().await.unwrap();
    let invt: Inventory = match sqlx::query_as("SELECT * FROM inventory WHERE id=$1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
    {
        Ok(i) => i,
        Err(e) => {
            tx.rollback().await.unwrap();
            eprintln!("error: {:?}", e);
            return;
        }
    };

    println!("#{} {:?}", i, invt);

    if invt.stock <= 0 {
        tx.rollback().await.unwrap();
        println!("#{} 库存不足", i);
        return;
    }

    // 不使用乐观锁，会导致超售，即 sock 变成负数
    // if let Err(e) = sqlx::query("UPDATE inventory SET stock = stock - 1 WHERE id=$1")
    //     .bind(invt.id)
    //     .execute(&mut *tx)
    //     .await
    // {
    //     tx.rollback().await.unwrap();
    //     eprintln!("error: {:?}", e);
    //     return;
    // };

    if let Err(e) =
        sqlx::query("UPDATE inventory SET stock = stock - 1, ver = $2 + 1 WHERE id=$1 AND ver=$2")
            .bind(invt.id)
            .bind(invt.ver)
            .execute(&mut *tx)
            .await
    {
        tx.rollback().await.unwrap();
        eprintln!("error: {:?}", e);
        return;
    };
    tx.commit().await.unwrap();
}

async fn find(conn: &sqlx::PgPool, id: i32) -> Result<Inventory, sqlx::Error> {
    sqlx::query_as("SELECT * FROM inventory WHERE id =$1")
        .bind(id)
        .fetch_one(conn)
        .await
}
