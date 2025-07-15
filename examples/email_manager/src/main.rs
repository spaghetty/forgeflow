fn main() {
    let trigger = GmailChecker::from_pool(); //define a loop time cadence
    let model = Gemini::new();
    let actuator = GmailActions::from_action(Action::MarkRead);
    let agent = Agent::new()
        .with_trigger(trigger)
        .with_model(model)
        .with_actuators(vec![actuator]);

    agent.run()?;
}
