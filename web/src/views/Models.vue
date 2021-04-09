<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Models</h1>
      </b-col>
    </b-row>

    <hr />

    <model-card
      v-for="(model, index) in model_data"
      :key="index"
      :model="model"
      :i="index"
    />

    <b-row class="justify-content-center">
      <b-col xs="12" lg="6">
        <b-button
          block
          class="mb-4 shadow"
          v-b-toggle.collapse-new
          variant="primary"
          style="border: none"
          onfocus="this.blur();"
        >
          <b-icon-plus-circle-fill></b-icon-plus-circle-fill>
<<<<<<< HEAD:web/src/views/Models.vue
          Add New Model
=======
          Register A New Model
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
        </b-button>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
<<<<<<< HEAD:web/src/views/Models.vue
      <b-col lg="7">
=======
      <b-col xs="12" lg="9">
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
        <b-collapse id="collapse-new" class="mb-4 nodeExpansion">
          <b-card
            class="mb-4 shadow"
            no-body
            style="border: none"
            onfocus="this.blur();"
          >
            <b-card-body title-tag="h5">
<<<<<<< HEAD:web/src/views/Models.vue
              <b-card-title>Connect a New Model</b-card-title>
              <p>
                Download our Python Module from PyPI
=======
              <b-card-title>Register A New Model</b-card-title>
              <p>
                1) Clone the Sybl Client repository
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
              </p>
              <b-row class="justify-content-center">
                <b-col >
                  <b-card class="shadow">
                    <b-row>
<<<<<<< HEAD:web/src/views/Models.vue
                      <b-col xl="11">
                        <code>{{ cli_deps }}</code>
                        <br />
                        <code>{{ cli_code }}</code>
=======
                      <b-col md="10">
                        <code>{{ clone_code }}</code>
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
                      </b-col>
                      <b-col style="text-align: right" lg="1">
                        <b-button
                          no-body
                          size="sm"
                          variant="dark"
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            @click="copy(clone_code)"
                          ></b-icon-clipboard-plus>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
              <p>
<<<<<<< HEAD:web/src/views/Models.vue
                Run the following command to add a new Model
=======
                2) Install the necessary requirements
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
              </p>
              <b-row class="justify-content-center">
                <b-col >
                  <b-card class="shadow">
                    <b-row>
<<<<<<< HEAD:web/src/views/Models.vue
                      <b-col xl="11">
                        <code>{{ cli_setup }}</code>
=======
                      <b-col md="10">
                        <code>{{ req_code }}</code>
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
                      </b-col>
                      <b-col style="text-align: right" lg="1">
                        <b-button
                          no-body
                          size="sm"
                          variant="dark"
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            @click="copy(req_code)"
                          ></b-icon-clipboard-plus>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
              <p>
                3) Copy your unique Private Key into your .env file
              </p>
              <p>
                4) Run the authentication script using your Sybl account details
              </p>
              <p>
                5) Now a new model is linked to Sybl and can be seen in the model dashboard, you must unlock it to continue,
              </p>
              <vue-markdown>**Inline Math**: $\sqrt{3x-1}+(1+x)^2$</vue-markdown>
            </b-card-body>
          </b-card>
        </b-collapse>
      </b-col>
    </b-row>
    <speedometer />
  </b-container>
</template>

<script>
import ModelCard from "@/components/ModelCard";
import VueMarkdown from 'vue-markdown';
import Speedometer from "@/components/charts/Speedometer";

export default {
  name: "Models",
  data() {
    return {
      auth_token: "",
      error: false,
<<<<<<< HEAD:web/src/views/Models.vue
      cli_deps: "pip install pandas pyOpenSSL python-dotenv xdg numpy zenlog",
      cli_code: "pip install -i https://test.pypi.org/simple/ syblmallus",
      cli_setup: "python3 -m sybl authenticate",
=======
      clone_code: "git clone https://github.com/G-Kemp101/mallus.git",
      req_code: "pip3 install -r requirements.txt",
      cli_setup: "sybl-cli new",
>>>>>>> Experiment with markdown parser and speedometer:web/src/views/Nodes.vue
    };
  },
  components: {
    ModelCard,
    VueMarkdown,
    Speedometer,
  },
  async created() {
    this.$store.dispatch("getModels");
  },
  methods: {
    async copy(s) {
      await navigator.clipboard.writeText(s);
    },
  },
  computed: {
    validation() {
      return !this.error;
    },
    model_data() {
      return this.$store.state.models.models;
    },
  },
};
</script>
