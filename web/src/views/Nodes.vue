<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Models</h1>
      </b-col>
    </b-row>

    <hr />

    <model-card
      v-for="(m, index) in model_data"
      :key="index"
      :data="m"
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
          Register A New Model
        </b-button>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="9">
        <b-collapse id="collapse-new" class="mb-4 nodeExpansion">
          <b-card
            class="mb-4 shadow"
            no-body
            style="border: none"
            onfocus="this.blur();"
          >
            <b-card-body title-tag="h5">
              <b-card-title>Register A New Model</b-card-title>
              <p>
                1) Clone the Sybl Client repository
              </p>
              <b-row class="justify-content-center">
                <b-col xs="12" lg="10">
                  <b-card class="shadow">
                    <b-row>
                      <b-col md="10">
                        <code>{{ clone_code }}</code>
                      </b-col>
                      <b-col style="text-align: right" md="2">
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
                2) Install the necessary requirements
              </p>
              <b-row class="justify-content-center">
                <b-col xs="12" lg="10">
                  <b-card class="shadow">
                    <b-row>
                      <b-col md="10">
                        <code>{{ req_code }}</code>
                      </b-col>
                      <b-col style="text-align: right" md="2">
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
  name: "Nodes",
  data() {
    return {
      model_data: [],
      auth_token: "",
      error: false,
      clone_code: "git clone https://github.com/G-Kemp101/mallus.git",
      req_code: "pip3 install -r requirements.txt",
      cli_setup: "sybl-cli new",
    };
  },
  components: {
    ModelCard,
    VueMarkdown,
    Speedometer,
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let data = await this.$http.get(
        `api/clients/models`
      );
      this.model_data = data.data;
    } catch (err) {
      console.log(err);
    }
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
  },
};
</script>
