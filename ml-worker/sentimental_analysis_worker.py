import uuid
import nltk
from nltk.sentiment import SentimentIntensityAnalyzer
import repository
import utils

nltk.download('vader_lexicon')
sia = SentimentIntensityAnalyzer()

def handle_sentimental_analysis(article_uuid: uuid.UUID) -> utils.Status:
    article = repository.get_article_by_uuid(article_uuid)

    if article == None:
        print("Warning: got invalid article UUID, throwing away message:", article_uuid)
        return utils.Status.INVALID

    html_content, title, language = article

    if html_content == None:
        print("Warning: got article without html_content, throwing away message:", article_uuid)
        return utils.Status.INVALID

    if language[0:2] != "en":
        print("Warning: got article with language that's not english (",language,"), dropping:", article_uuid)
        return utils.Status.INVALID

    text_content = utils.extract_text_from_html_content(html_content)
    
    pol_scores = sia.polarity_scores(text_content)

    db_status = repository.upsert_article_data_unsafe(
        article_id=article_uuid, 
        sentiment_score=pol_scores["compound"]
    )

    print(f"Calculated successfully sentimental score {pol_scores["compound"]} for {article_uuid}")

    if db_status:
        return utils.Status.SUCCESS
    
    return utils.Status.FAILED