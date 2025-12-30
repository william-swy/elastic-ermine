pub mod api {
    #[derive(Debug, Clone)]
    pub struct Message {

    }

    #[derive(Debug, Default)]
    pub struct APIView {

    }

    impl APIView {
        pub fn view(&self) -> iced::Element<'_, Message> {
            iced::widget::text("API Invocations WIP").into()
        }
    }
}