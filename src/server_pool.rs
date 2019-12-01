#[derive(Debug)]
struct Backend {
    url: String,
    alive: bool,
    // mux:
    // reverse_proxy

    // URL          *url.URL
    // Alive        bool
    // mux          sync.RWMutex
    // ReverseProxy *httputil.ReverseProxy
}

#[derive(Debug)]
struct ServerPool {
    backends: Vec<Backend>,
    current: usize, // backends []*Backend
                    // current  uint64
}