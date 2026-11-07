use super::AnalysisIntent;

pub(super) fn classify(question: &str) -> AnalysisIntent {
    let question = question.to_ascii_lowercase();
    if [
        "guaranteed return",
        "guaranteed profit",
        "sure-shot",
        "sure shot",
        "risk-free profit",
        "should i buy",
        "should i sell",
        "buy this stock",
        "sell this stock",
        "buy or sell",
    ]
    .iter()
    .any(|term| question.contains(term))
    {
        AnalysisIntent::SafetyBoundary
    } else if ["why did", "why is", "reason for", "cause of"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Causality
    } else if ["compare", "versus", " vs "]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Comparison
    } else if ["fundamental", "revenue", "profitability", "news", "company"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::CompanyResearch
    } else if ["forecast", "predict", "target price", "next price"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Forecast
    } else if ["rsi", "sma", "atr", "indicator", "moving average"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Indicators
    } else if ["drawdown", "risk", "volatile", "volatility"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Risk
    } else if ["trend", "direction", "momentum"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::Trend
    } else if ["return", "change", "performance", "gain", "loss"]
        .iter()
        .any(|term| question.contains(term))
    {
        AnalysisIntent::PriceChange
    } else {
        AnalysisIntent::DatasetSummary
    }
}
