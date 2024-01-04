#[cfg(test)]
mod test {
    extern crate grapefruit;

    use chrono::{DateTime, Utc};

    use grapefruit::{BaseRepository, Value};
    use grapefruit_macros::GrapefruitTable;

    #[derive(Debug, Default, Clone, GrapefruitTable)]
    #[table(name = "t_user", curd = "true")]
    pub struct User {
        #[id(name = "id", id_type = "generator")]
        pub id: Option<i64>,
        #[column(name = "name")]
        pub name: String,
        #[column(name = "password")]
        pub password: String,
        #[column(name = "addr", insert_strateg = "not_null")]
        pub addr: Option<String>,
        #[column(name = "created_at", update_strateg = "never", fill = "insert")]
        pub created_at: Option<DateTime<Utc>>,
        #[column(name = "updated_at", fill = "insert_and_update", version = "true")]
        pub updated_at: Option<DateTime<Utc>>,
        #[column(name = "deleted", select = "false", is_logic_delete = "true", fill = "insert")]
        pub deleted: Option<bool>,
    }

    pub struct UserRepository {}

    impl UserRepository {
        pub fn new() -> Self {
            UserRepository {}
        }
    }

    impl BaseRepository<i64, User> for UserRepository {}

    #[derive(Default)]
    pub struct CustomMetaObjectHandler {}

    impl grapefruit::MetaObjectHandler for CustomMetaObjectHandler {
        fn insert_fill(&self, meta: &mut grapefruit::MetaObject) {
            meta.set_insert_fill(
                "created_at",
                Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
            );
            meta.set_insert_fill(
                "updated_at",
                Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
            );
            meta.set_insert_fill("deleted", Value::Bool(Some(false)));
        }

        fn update_fill(&self, meta: &mut grapefruit::MetaObject) {
            meta.set_insert_fill(
                "updated_at",
                Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))),
            );
        }
    }
    mod aaa {

        use grapefruit::BaseRepository;
        use grapefruit::Column;
        use grapefruit::Entity;
        use grapefruit::Grapefruit;
        use grapefruit::GrapefruitOptions;
        use grapefruit::GrapefruitRepository;
        use grapefruit::Wrapper;
        use sqlx::types::chrono::Utc;

        use crate::test::CustomMetaObjectHandler;
        use crate::test::User;
        use crate::test::UserDef;
        use crate::test::UserRepository;
        // use crate::test::UserIden;

        #[test]
        fn test() {
            // println!("{:?}", User::table_name());
            println!("{:?}", User::insert_columns());
            println!("{:?}", User::update_columns());
            println!("{:?}", User::select_columns());
            let user = User {
                id: Default::default(),
                name: "user_1".into(),
                password: "password".into(),
                addr: None,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                deleted: Some(false),
            };

            let a = &user;
            let b = a.clone();
            let value = b.to_value();
            println!("{:?}", value);

            println!("{:?}", User::columns());
            let id = UserDef::Id;

            println!("{:?}", id.column_info());
            // println!("{:?}", id.is_table_id());
        }

        #[tokio::test]
        async fn test_insert() {
            let config =
                GrapefruitOptions::new("postgres://postgres:123456@127.0.0.1:5432/postgres")
                    .with_meta_object_handler(Box::new(CustomMetaObjectHandler::default()));
            let mut grapefruit = Grapefruit::new(&config);
            grapefruit.init().await.ok();
            grapefruit::GRAPEFRUIT.set(grapefruit.clone()).ok();

            let user = User {
                id: Default::default(),
                name: "user_1".into(),
                password: "password".into(),
                addr: None,
                created_at: None,
                updated_at: None,
                deleted: None,
            };
            // User::add(&user, &mut grapefruit).await.unwrap();
            // grapefruit.insert(&user).await.unwrap();
            grapefruit.insert(&user).await.unwrap();
        }

        #[tokio::test]
        async fn test_insert_batch() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();

            let user1 = User {
                id: Default::default(),
                name: "user_1".into(),
                password: "password".into(),
                addr: None,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                deleted: Some(false),
            };
            let user2 = User {
                id: Default::default(),
                name: "user_2".into(),
                password: "password".into(),
                addr: None,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                deleted: Some(false),
            };
            // User::add(&user, &mut grapefruit).await.unwrap();
            grapefruit
                .insert_batch(&vec![&user1, &user2])
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_select() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let res = grapefruit
                .select_by_id::<User, i64>(577109878368243712)
                .await
                .unwrap();
            println!("res:{:?}", res);
            grapefruit::GRAPEFRUIT.set(grapefruit).ok();
            let r = UserRepository::new();
            let res2 = r.select_by_id(577109878368243712).await.unwrap();
            println!("res:{:?}", res2);
        }

        #[tokio::test]
        async fn test_select_page() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let wrapper = Wrapper::new();
            // .eq(UserDef::Id, 577109878422769665 as i64)
            // .like_left(UserDef::Password, "word")
            // .between(
            //     UserDef::Id,
            //     577109878422769664 as i64,
            //     577109878422769666 as i64,
            // )
            // .in_list(
            //     UserDef::Id,
            //     vec![577109878422769665 as i64, 577109878422769666 as i64],
            // )
            // .and_fn(|w| {
            //     w.like(UserDef::Name, "1")
            //         .or()
            //         .like_right(UserDef::Name, "user")
            // });
            let res = grapefruit
                .page_by_wrapper::<User>(1, 2, wrapper)
                .await
                .unwrap();
            println!("res:{:?}", res);
        }

        #[tokio::test]
        async fn test_count_all() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let res = grapefruit.count_all::<User>().await.unwrap();
            println!("res:{:?}", res);
        }

        #[tokio::test]
        async fn test_select_wrapper() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let wrapper = Wrapper::new()
                .eq(UserDef::Id, 577109878422769665 as i64)
                .like_left(UserDef::Password, "word")
                .between(
                    UserDef::Id,
                    577109878422769664 as i64,
                    577109878422769666 as i64,
                )
                .in_list(
                    UserDef::Id,
                    vec![577109878422769665 as i64, 577109878422769666 as i64],
                )
                .and_fn(|w| {
                    w.like(UserDef::Name, "1")
                        .or()
                        .like_right(UserDef::Name, "user")
                });
            let res = grapefruit.select_by_wrapper::<User>(wrapper).await.unwrap();
            println!("res:{:?}", res);
        }

        #[tokio::test]
        async fn test_delete_by_id() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let res = grapefruit
                .delete_by_id::<User, i64>(584675103976067073)
                .await
                .unwrap();
            println!("res:{:?}", res);
        }
        #[tokio::test]
        async fn test_delete_by_ids() {
            let mut grapefruit = Grapefruit::new(&GrapefruitOptions::new(
                "postgres://postgres:123456@127.0.0.1:5432/postgres",
            ));
            grapefruit.init().await.ok();
            let ids = vec![584675103976067073];
            let res = grapefruit.delete_by_ids::<User, i64>(&ids).await.unwrap();
            println!("res:{:?}", res);
        }

        #[tokio::test]
        async fn test_update_by_id() {
            let config =
                GrapefruitOptions::new("postgres://postgres:123456@127.0.0.1:5432/postgres")
                    .with_meta_object_handler(Box::new(CustomMetaObjectHandler::default()));
            let mut grapefruit = Grapefruit::new(&config);
            grapefruit.init().await.ok();
            grapefruit::GRAPEFRUIT.set(grapefruit.clone()).ok();

            let user = User {
                id: Some(577109878368243712),
                name: "user_5".into(),
                password: "password5".into(),
                addr: None,
                created_at: None,
                updated_at: None,
                deleted: None,
            };
            let res = grapefruit.update_by_id::<User>(&user).await.unwrap();
            println!("res:{:?}", res);
        }

        #[tokio::test]
        async fn test_wrapper() {
            let wrapper = Wrapper::new()
                .eq(UserDef::Id, 577109878368243712 as i64)
                .between(UserDef::CreatedAt, Utc::now(), Utc::now())
                .like_left(UserDef::Name, "123")
                .in_list(UserDef::Id, vec![1, 2, 3])
                .and_fn(|w| {
                    w.eq(UserDef::Name, "user_1")
                        .or()
                        .eq(UserDef::Name, "user_2")
                })
                .build(&grapefruit::Platform::Postgres, 1);

            println!("{:?}", wrapper.0);
            // println!("{:?}", wrapper.1);
        }
    }
}
