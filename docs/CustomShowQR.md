自定义打印二维码
==============

用于将二维码TCP传输等。作者暂时没有遇到此类场景，并不确定是否应用于所有场景，或许这里的Pin应该改成Box<dyn trait>。

  ```
  .show_rq(ShowQR::Custom(Box::pin(|buff| {  // 自定义显示二维码
    Box::pin(async move {
         println!("buff : {:?}", buff.to_vec());
         Ok(())
  })
  ```