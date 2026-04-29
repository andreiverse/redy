from transformers import pipeline, AutoModelForSequenceClassification, AutoTokenizer
import torch

# Using a much more accurate model for zero-shot classification
MODEL_ID = "MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli"

_classifier = None

def get_classifier():
    global _classifier
    if _classifier is None:
        print(f"Loading model {MODEL_ID} and applying dynamic quantization...")
        
        # Explicitly load in float32 to avoid "Float vs Half" errors on CPU
        model = AutoModelForSequenceClassification.from_pretrained(
            MODEL_ID, 
            torch_dtype=torch.float32
        )
        tokenizer = AutoTokenizer.from_pretrained(MODEL_ID)

        # Ensure model is in float mode before quantization
        model.float()

        # Apply dynamic quantization to the Linear layers
        quantized_model = torch.quantization.quantize_dynamic(
            model, 
            {torch.nn.Linear}, 
            dtype=torch.qint8
        )

        _classifier = pipeline(
            "zero-shot-classification",
            model=quantized_model,
            tokenizer=tokenizer,
            device=-1 # Ensure CPU
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
