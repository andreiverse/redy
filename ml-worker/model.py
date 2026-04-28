from transformers import pipeline
import torch

# Using a much more accurate model for zero-shot classification
MODEL_ID = "MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli"

_classifier = None

def get_classifier():
    global _classifier
    if _classifier is None:
        print(f"Loading model {MODEL_ID}...")
        # Use CPU for end devices/small footprint
        _classifier = pipeline(
            "zero-shot-classification",
            model=MODEL_ID,
            device="cpu"
        )
    return _classifier

def classify(text: str, candidate_labels: list, hypothesis_template: str = None):
    classifier = get_classifier()
    
    kwargs = {"multi_label": False}
    if hypothesis_template:
        kwargs["hypothesis_template"] = hypothesis_template

    # Truncate text to avoid model max length issues
    result = classifier(text[:250], candidate_labels, **kwargs)
    return result
