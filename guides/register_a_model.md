# Register a Model

This guide will help you register a ML model with Sybl. Make sure you have
registered yourself as a client and have a copy of your private key. Look at
the guide Become a Client with Sybl for further guidance.

### The Requirements

To register a model, you'll need:

- Python 3.6 or later
- Our Client-Model Interface
  ([mallus](https://test.pypi.org/project/sybl-test/)) installed. To do this:
  - Run `pip3 install -i https://test.pypi.org/simple/ syblmallus
  --extra-index-url https://pypi.org/simple/` to install mallus
- A machine learning model implemented in Python
- A computer capable of connecting to the Sybl service and processing possibly
  large datasets

### Getting Started

1. Create a new `.env` file in the same directory as your model, containing
   your private key:

    ```bash
    PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----
    <Your private key>
    -----END RSA PRIVATE KEY-----
    "
    ```

2. Run the authentication script: `python3 -m sybl authenticate` in the
   directory with your `.env` file. This will ask you for your account's email
   and password as well as the name of model you're registering (Optionally
   provide the email and model name through `--email` and `--modelname`
   arguments respectively). This will authenticate and register your model to
   your account. However, you will need to unlock your model on the Sybl
   website before it can be used in a job.
3. Go to the Sybl website and navigate to your client dashboard by selecting
   *Models*

    ![images/client_dropdown.png](images/client_dropdown.png)

4. You will see your newly added model which is currently locked. Select the
   model and enter your password to unlock the model.

    ![images/unlock_model.png](images/unlock_model.png)

5. You are now ready to start using your model. Run your model using a script
   which registers your callback, loads your model and runs `sybl.connect()` -
   see examples below.
6. Once you've connected your model, you'll be able to see your model's
   performance through your dashboard.

### Using your model

These are a few functions you should be aware of when trying to connect a
model:

**Sybl(***email: str = None, model_name: str = None, callback: Callable = None,
job_config = JobConfig()* **)**

This instantiates a new Sybl object which will be used to connect to the Sybl
service. Optionally provide email and model_name to register a model on
instantiation.  Will call sybl.load_model(email, model_name). Optionally
provide the callback function. job_config can be specified if not default will
be used.

sybl.**load_model(***email: str, model_name: str***)**

This will load this model's access token which will be stored in
`xdg_data_home` by default.

sybl.**register_callback(***callback: Callable[[pd.DataFrame, pd.DataFrame], pd.DataFrame]***)**

Registers the callback function used for processing data. The callback must
take in a training dataset and prediction dataset as two `pandas.DataFrame`
objects, as well as a prediction type string `"regression"` or
`"classification"` . The callback must return a single column
`pandas.DataFrame` which is the predicted values for the prediction dataset.

sybl.**load_config(***config: JobConfig***)**

Loads a given `JobConfig` into the Sybl object. This is used to decide whether
a job should be accepted or rejected given its configuration, including details
such as the job's timeout, size, type and dimensionality.

sybl.**set_comparison_function(***comparison_function: Callable[[Dict], bool]***)**

Sets a custom comparison function to use for incoming job configurations from
the distributed control layer. Given a job's config details, the function
should return True if you want your model to run the job and False if you want
your model to reject the job. This allows you to override the default handling
and implement more complex logic to decide whether a job should be accepted.
There are no penalties for rejecting a job.

For example, you may want a different timeout for classification and regression
problems, which the default handler does not allow for. By registering a custom
function to handle incoming requests, you are provided with all the data that
Sybl uses for job configurations and can manipulate it as you please.

sybl.**connect()**

Starts a connection with the Sybl service. Will connect to the service and wait
until a dataset is received. The job configuration will be compared to you
`JobConfig` and process if there is a match. The connection will continue until
forcibly closed.

**JobConfig(***prediction_types: List[str] = ["regressions", "classification"],
timeout: int = 10***)**

Creates and returns new `JobConfig` object which can be customised to accept or
reject jobs.

### Examples

1. Basic connection script with no `JobConfig` handler function

    ```python
    import pandas as pd  # type: ignore

    from sybl.client import Sybl
    from sybl.client import JobConfig

    sybl = Sybl()

    def callback(train: pd.DataFrame, predict: pd.DataFrame):
        """<Your ML MODEL goes here>"""

    sybl.register_callback(callback)
    sybl.load_model("<Your EMAIL goes here>", "<Your MODEL NAME goes here>")

    sybl.connect()
    ```

2. Connection script with a custom `JobConfig` handler function

    ```python
    from typing import Dict

    import pandas as pd  # type: ignore

    from sybl.client import Sybl
    from sybl.client import JobConfig

    sybl = Sybl()

    def callback(train: pd.DataFrame, predict: pd.DataFrame):
        """<Your ML MODEL goes here>"""

    def handle_incoming_configs(config: Dict) -> bool:
        # Extract the data contained in the config
        prediction_type = job_config["prediction_type"]
        message_creation_timestamp = job_config["message_creation_timestamp"]
        prediction_cutoff_timestamp = job_config["prediction_cutoff_timestamp"]

        # Calculate how long we have
        time_difference = prediction_cutoff_timestamp - message_creation_timestamp

        # Use 15 minutes for classification, 5 minutes for regression
        if prediction_type == "classification":
            return 15 * 60 <= time_difference
        else:
            return 5 * 60 <= time_difference

    sybl.register_callback(callback)
    sybl.load_model("<Your EMAIL goes here>", "<Your MODEL NAME goes here>")
    sybl.set_comparison_function(handle_incoming_configs)

    sybl.connect()
    ```
