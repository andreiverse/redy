from transformers import pipeline
import torch

# Using a highly efficient x-small model for fast CPU inference and low RAM usage
MODEL_ID = "MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33"

_classifier = None

def get_classifier():
    global _classifier
    if _classifier is None:
        print(f"Loading model {MODEL_ID}...")
        
        # This model is small (~140MB) and loads instantly on CPU
        _classifier = pipeline(
            "zero-shot-classification",
            model=MODEL_ID,
            device=-1 # Ensure CPU
        )
    return _classifier

def classify(text: str, candidate_labels: list, hypothesis_template: str = None):
    classifier = get_classifier()
    
    kwargs = {"multi_label": False}
    if hypothesis_template:
        kwargs["hypothesis_template"] = hypothesis_template

    # Truncate text to avoid model max length issues
    # Using inference_mode for better performance
    with torch.inference_mode():
        result = classifier(text[:250], candidate_labels, **kwargs)
    return result
