# Use Cases

## Development & Pairing

### Pair-debug a backend
Share port 3000, teammate's React app hits their `localhost:3000` instead of having to deploy to staging.

**Example:**
```bash
# Backend dev
rift share 3000 --secrets .env.rift

# Frontend dev
rift connect rift://... --request-secrets --save-secrets .env
npm run dev  # Uses backend dev's actual service
```

---

### "Works on my machine" issues
Let teammate access your *actual* running service to reproduce the bug.

---

### Frontend ↔ Backend pairing
Share your API, they develop against real data without mocking.

---

### Code review with context
Share your running branch, reviewer tests locally before approving PR.

---

## Database & Infrastructure

### Share a database
Share Postgres (5432), teammate uses `localhost:5432` — no config changes needed.

```bash
# Dev with DB
rift share 5432

# Teammate
rift connect rift://...
psql -h localhost -p 5432 -U user mydb
```

---

### Share Redis/cache
Share 6379, teammate's app connects without config change.

---

### Multi-service stack
Share API + DB + Cache as multiple ports, teammate has your entire stack.

---

## Demo & Collaboration

### Demo internal tools
Share admin dashboards to teammates without deploying to staging.

---

### OSS collaboration
Maintainer shares failing service, contributor debugs locally without complex setup.

---

### Cross-team debugging
Backend hosts "known good" state for frontend team to test against.

---

## Share Any Local Web UI

Run **any** local web interface and share it with teammates — no cloud deployment needed.

### Streamlit
```bash
streamlit run app.py  # → localhost:8501
rift share 8501

# Teammate
rift connect rift://... 
# → Their localhost:8501 shows YOUR Streamlit app
```

### Gradio
```bash
python app.py  # → localhost:7860
rift share 7860
```

### Jupyter
```bash
jupyter notebook  # → localhost:8888
rift share 8888
```

### FastAPI docs
```bash
uvicorn main:app  # → localhost:8000
rift share 8000
# Teammate accesses localhost:8000/docs
```

**Works with any local web UI** — teammate sees your app on their `localhost`, they can interact, test, debug. No Streamlit Cloud, no ngrok, no public exposure.

---

## GPU & Compute Resources

### Model server access
```bash
# GPU machine: run a model server
python -m vllm.entrypoints.openai.api_server --model mistral-7b
rift share 8000

# Laptop: use it as if local
rift connect rift://... 
# Now use http://localhost:8000 in your code
```

Your laptop talks to the GPU box as `localhost`. Run eval scripts, notebooks, anything — the heavy compute stays remote, but feels local.

---

### TensorBoard sharing
```bash
# Training machine
tensorboard --logdir=runs --port=6006
rift share 6006

# Team member
rift connect rift://...
# Open localhost:6006 in browser
```

---

### Distributed training observability
Share TensorBoard + Ray dashboard + training control endpoints. Keep the training job on a beefy box, but make observability local-to-local.
