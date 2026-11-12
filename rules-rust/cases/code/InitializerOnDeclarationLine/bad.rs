struct Writer;

impl Writer {
    const WINDOW: Duration =
        Duration::from_secs(60);

    fn write(&self, value: &Config) -> Result<(), ConfigurationError> {
        let content =
            serde_saphyr::to_string(value).map_err(|source| ConfigurationError::Serialize {
                path: self.path.clone(),
                source: Box::new(source),
            })?;
        let invoice_repository: Arc<dyn InvoiceRepository> =
            Arc::new(InMemoryInvoiceRepository::new());
        let Some(invoice) =
            find_invoice(id) else {
            return Ok(());
        };
        Ok(())
    }
}

const LIMIT: u32 =
    compute_limit();

static NAME: &str =
    "invoice";

trait Gate {
    const WINDOW: Duration =
        Duration::from_secs(60);
}

thread_local! {
    static CACHE: RefCell<Cache> =
        RefCell::new(Cache::new());
}

async fn watch(mut tick: Receiver<u32>) {
    tokio::select! {
        value = tick.recv() => {
            let total =
                compute(value);
        }
        else => {
            let total = compute_default();
        }
    }
}

fn collect() {
    let items = vec![
        {
            let first =
                compute();
            first
        },
        second,
    ];
}

fn run() {
    let feed = stream! {
        let total =
            compute();
        yield total;
    };
}

thread_local! {
    static LOCAL: RefCell<u32> =
        RefCell::new(0)
}

fn prepared() {
    m! {
        prepare();
        let total =
            compute()
    }
}

fn flagged() {
    m! {
        let ok =
            a || b
    }
}
