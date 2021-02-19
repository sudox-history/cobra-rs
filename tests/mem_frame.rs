use cobra_rs::mem::Frame;

#[tokio::test]
async fn simple_frame() {
    let data = vec![1_u8, 2, 3];
    let frame = Frame::create(1_u8, &data);

    assert_eq!(frame.to_vec(), vec![0_u8, 4, 1, 1, 2, 3]);
    assert_eq!(frame.get_body().to_vec(), vec![1_u8, 2, 3]);
}
