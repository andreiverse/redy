import uuid
import repository
import utils
import model

SENTIMENT_LABELS = ["positive", "negative"]

def handle_sentimental_analysis(article_uuid: uuid.UUID) -> utils.Status:
    article = repository.get_article_by_uuid(article_uuid)

    if article == None:
        print("Warning: got invalid article UUID, throwing away message:", article_uuid)
        return utils.Status.INVALID

    html_content, title, language, feed_id = article

    if html_content == None:
        print("Warning: got article without html_content, throwing away message:", article_uuid)
        return utils.Status.INVALID

    if language[0:2] != "en":
        print("Warning: got article with language that's not english (",language,"), dropping:", article_uuid)
        return utils.Status.INVALID

    text_content = utils.extract_text_from_html_content(html_content)
    
    # We use the same zero-shot model for sentiment
    result = model.classify(text_content, SENTIMENT_LABELS)
    
    # Calculate a score between -1 and 1
    # labels are sorted by score in result['labels'] and result['scores']
    scores_dict = dict(zip(result['labels'], result['scores']))
    sentiment_score = scores_dict['positive'] - scores_dict['negative']

    db_status = repository.upsert_article_data_unsafe(
        article_id=article_uuid, 
        sentiment_score=float(sentiment_score)
    )

    print(f"Calculated successfully sentimental score {sentiment_score:.2f} for {article_uuid}")

    if db_status:
        return utils.Status.SUCCESS
    
    return utils.Status.FAILED
