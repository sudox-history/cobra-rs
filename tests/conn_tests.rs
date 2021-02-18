// use cobra_rs::transport::listener::Listener;
// use cobra_rs::transport::conn::Conn;
// use cobra_rs::transport::frame::Frame;
// use std::sync::Arc;
//
// #[tokio::test]
// async fn single_conn() {
//     const ADDR: &str = "127.0.0.1:5000";
//     const KIND_A: u8 = 1;
//
//     let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         let frame = Frame::new(KIND_A, vec![1, 2, 3]);
//         assert!(conn.write(frame).await.is_ok());
//     });
//
//     let conn = listener.accept().await.unwrap();
//
//     assert_eq!(conn.read(KIND_A).await.unwrap().get_data(), vec![1, 2, 3]);
// }
//
// #[tokio::test]
// async fn many_package() {
//     const ADDR: &str = "127.0.0.1:5001";
//     const KIND_A: u8 = 1;
//
//     let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         for i in 0..1000 {
//             let body: Vec<u8> = (i..(i+1000)).map(|x| x as u8).collect();
//             let frame = Frame::new(KIND_A, body);
//             assert!(conn.write(frame).await.is_ok());
//         }
//     });
//
//     let conn = listener.accept().await.unwrap();
//
//     for i in 0..1000 {
//         let body: Vec<u8> = (i..(i+1000)).map(|x| x as u8).collect();
//         let frame = conn.read(KIND_A).await.unwrap().get_data();
//         assert_eq!(frame, body);
//     }
// }
//
// #[tokio::test]
// async fn difference_kind() {
//     const ADDR: &str = "127.0.0.1:5002";
//     const KIND_A: u8 = 1;
//     const KIND_B: u8 = 2;
//
//     let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn_a = Arc::new(Conn::connect(ADDR).await.unwrap());
//         let conn_b = conn_a.clone();
//
//         tokio::spawn(async move {
//             let frame = Frame::new(KIND_A, vec![1, 2, 3]);
//             assert!(conn_a.write(frame).await.is_ok());
//         });
//
//         tokio::spawn(async move {
//             let frame = Frame::new(KIND_B, vec![3, 2, 1]);
//             assert!(conn_b.write(frame).await.is_ok());
//         });
//     });
//
//     let conn = listener.accept().await.unwrap();
//
//     assert_eq!(conn.read(KIND_A)
//                    .await
//                    .unwrap()
//                    .get_data(), vec![1, 2, 3]);
//     assert_eq!(conn.read(KIND_B)
//                    .await
//                    .unwrap()
//                    .get_data(), vec![3, 2, 1]);
// }
//
// #[tokio::test]
// async fn many_connections() {
//     const ADDR: &str = "127.0.0.1:5003";
//     const KIND_A: u8 = 1;
//
//     let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         let frame = Frame::new(KIND_A, vec![1]);
//         assert!(conn.write(frame).await.is_ok());
//     });
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         let frame = Frame::new(KIND_A, vec![2]);
//         assert!(conn.write(frame).await.is_ok());
//     });
//
//     let conn_a = listener.accept().await.unwrap();
//     let conn_b = listener.accept().await.unwrap();
//
//     let mut count = conn_a.read(KIND_A).await.unwrap().get_data()[0];
//     count += conn_b.read(KIND_A).await.unwrap().get_data()[0];
//
//     assert_eq!(count, 3);
// }
//
// #[tokio::test]
// async fn read_and_write() {
//     const ADDR: &str = "127.0.0.1:5004";
//     const KIND_A: u8 = 1;
//
//     let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         let frame = conn.read(KIND_A).await.unwrap();
//         assert_eq!(frame.get_data(), vec![1, 2, 3]);
//
//         let frame = Frame::new(KIND_A, vec![3, 2, 1]);
//         assert!(conn.write(frame).await.is_ok());
//     });
//
//     let conn = listener.accept().await.unwrap();
//
//     let frame = Frame::new(KIND_A, vec![1, 2, 3]);
//     assert!(conn.write(frame).await.is_ok());
//
//     let frame = conn.read(KIND_A).await.unwrap();
//     assert_eq!(frame.get_data(), vec![3, 2, 1]);
// }
//
// #[tokio::test]
// async fn close_test() {
//     const ADDR: &str = "127.0.0.1:5005";
//     const KIND_A: u8 = 1;
//
//    let listener = Listener::listen(ADDR).await.unwrap();
//
//     tokio::spawn(async {
//         let conn = Conn::connect(ADDR).await.unwrap();
//         let frame = Frame::new(KIND_A, vec![1, 2, 3]);
//         assert!(conn.write(frame).await.is_ok());
//     });
//
//     let conn = listener.accept().await.unwrap();
//     listener.close_all_connections().await;
//
//     assert!(conn.read(KIND_A).await.is_none());
// }
