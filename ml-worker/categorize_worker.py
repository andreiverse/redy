import uuid
import repository
import utils
import model

# Mapping descriptive labels to database categories
# More distinct labels help the model avoid overlapping concepts
CATEGORY_MAPPING = {
    "political news and government policy": "Politics",
    "software, AI, gadgets and tech industry": "Technology",
    "stock markets, companies and economy": "Business",
    "professional sports and athletic competitions": "Sports",
    "movies, celebrities, music and pop culture": "Entertainment",
    "medical news and public health": "Health",
    "scientific research and discoveries": "Science",
    "crime, accidents, and general news": "General"
}

def handle_categorize(article_uuid: uuid.UUID) -> utils.Status:
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
    
    # Better text preparation: Title + first few paragraphs
    paragraphs = [p.strip() for p in text_content.split('\n') if p.strip()][:3]
    combined_text = f"Headline: {title}\n\nSummary: {title}\n\n" + '\n'.join(paragraphs)

    candidate_labels = list(CATEGORY_MAPPING.keys())
    
    # Use hypothesis template for better results
    result = model.classify(
        combined_text[:1500], 
        candidate_labels,
        hypothesis_template="This article is about {}."
    )

    best_label = result["labels"][0]
    category = CATEGORY_MAPPING[best_label]

    # Better threshold: if the model isn't confident, it's likely General News
    if result["scores"][0] < 0.3:
        category = "General"

    db_status = repository.upsert_article_data_unsafe(
        article_id=article_uuid, 
        category=category
    )

    print(f"Categorized as {category} (ML label: '{best_label}', score: {result['scores'][0]:.2f}) for {article_uuid}")

    if db_status:
        return utils.Status.SUCCESS

    return utils.Status.FAILED

