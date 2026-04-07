from typing import Optional
import uuid

import psycopg2

from config import POSTGRES_URL

conn = psycopg2.connect(POSTGRES_URL)

def get_article_by_uuid(id: uuid.UUID):
    cursor = conn.cursor()
    try:
        cursor.execute(
            "SELECT html_content, title, language FROM article WHERE id = %s",
            (str(id),)
        )
        return cursor.fetchone()

    except Exception as e:
        conn.rollback()
        print("DB error:", e)
        return None

    finally:
        cursor.close()

def upsert_article_data_unsafe(article_id: uuid.UUID, **kwargs) -> bool:
    """
    Insert or update the article_data row.
    Only updates fields that are not None.
    
    Example usage:
        upsert_article_data(article_id, sentiment_label="positive", sentiment_score=0.8)
        upsert_article_data(article_id, readability_score=72.5)
    """
    if not kwargs:
        return False  # Nothing to update

    # Separate columns and values that are not None
    columns = []
    values = []
    updates = []

    for key, value in kwargs.items():
        if value is not None:
            columns.append(key)
            values.append(value)
            updates.append(f"{key} = EXCLUDED.{key}")

    if not columns:
        return False  # Nothing to update

    # Add id column
    columns.insert(0, "id")
    values.insert(0, str(article_id))
    
    sql = f"""
    INSERT INTO article_data ({', '.join(columns)})
    VALUES ({', '.join(['%s'] * len(values))})
    ON CONFLICT (id) DO UPDATE
    SET {', '.join(updates)}
    """

    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, tuple(values))
        conn.commit()
        return True
    except Exception as e:
        conn.rollback()
        print("DB error in upsert_article_data:", e)
        return False