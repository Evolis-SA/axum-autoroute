use axum_autoroute::autoroute;
use axum;
use utoipa;

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn wrong_status_1() -> WrongStatus1Responses {
    12i32.into_not_found()
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn wrong_status_2() -> WrongStatus2Responses {
    i32::into_not_found(12i32)
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn wrong_status_3() -> WrongStatus3Responses {
    WrongStatus3Responses::NotFound(12i32)
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn missing_use_trait_1() -> MissingUseTrait1Responses {
    12i32.into_im_a_teapot()
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn missing_use_trait_2() -> MissingUseTrait2Responses {
    i32::into_im_a_teapot(12i32)
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn bad_type_1() -> BadType1Responses {
    12u32.into_im_a_teapot()
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn bad_type_2() -> BadType2Responses {
    u32::into_im_a_teapot(12u32)
}

/// doc
#[autoroute(GET, path="/hello", responses=[(IM_A_TEAPOT, body=i32, description="hello")])]
pub fn bad_type_3() -> BadType3Responses {
    BadType3Responses::ImATeapot(12u32)
}

fn main() {}