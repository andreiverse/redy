import uuid
import repository
import utils
import model

def handle_categorize(article_uuid: uuid.UUID) -> utils.Status:
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
    
    # Better text preparation: Title + first few paragraphs
    paragraphs = [p.strip() for p in text_content.split('\n') if p.strip()][:3]
    combined_text = f"Headline: {title}\n\nSummary: {title}\n\n" + '\n'.join(paragraphs)

    # Get categories for this feed, or all categories if none assigned to feed
    db_categories = repository.get_feed_categories(feed_id)
    if not db_categories:
        db_categories = repository.get_all_categories()
    
    if not db_categories:
        print(f"Warn: No categories found in database for article {article_uuid}")
        return utils.Status.SUCCESS

    # Map model_description -> category_id
    cat_mapping = {desc: cid for cid, desc in db_categories}
    candidate_labels = list(cat_mapping.keys())
    
    # Use hypothesis template for better results
    result = model.classify(
        combined_text[:1500], 
        candidate_labels,
        hypothesis_template="This article is about {}."
    )

    best_label = result["labels"][0]
    category_id = cat_mapping[best_label]

    # Better threshold: if the model isn't confident, it's likely General News
    if result["scores"][0] < 0.3:
        # Try to find a "General" category if it exists
        general_id = next((cid for cid, desc in db_categories if "general" in desc.lower()), None)
        if general_id:
            category_id = general_id

    db_status = repository.upsert_article_data_unsafe(
        article_id=article_uuid, 
        category_id=category_id
    )

    print(f"Categorized as {category_id} (ML label: '{best_label}', score: {result['scores'][0]:.2f}) for {article_uuid}")

    if db_status:
        return utils.Status.SUCCESS

    return utils.Status.FAILED
