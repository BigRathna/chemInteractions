from optimum.onnxruntime import ORTModelForSeq2SeqLM
from transformers import AutoTokenizer
import os

MODEL_ID = "sagawa/ReactionT5-product-prediction"
OUTPUT_DIR = "../models/"
os.makedirs(OUTPUT_DIR, exist_ok=True)

print("Exporting encoder and decoder to ONNX...")
model = ORTModelForSeq2SeqLM.from_pretrained(
    MODEL_ID,
    export=True,
    provider="CPUExecutionProvider"
)
tokenizer = AutoTokenizer.from_pretrained(MODEL_ID)

model.save_pretrained(OUTPUT_DIR)
tokenizer.save_pretrained(OUTPUT_DIR)

print(f"Saved to {OUTPUT_DIR}")
print("Files:")
for f in os.listdir(OUTPUT_DIR):
    f_path = os.path.join(OUTPUT_DIR, f)
    if os.path.isfile(f_path):
        size = os.path.getsize(f_path) / 1e6
        print(f"  {f}  ({size:.1f} MB)")
