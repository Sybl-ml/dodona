<template>
  <b-container fluid="md">
    <b-row> 
      <b-col>
        <h1>Nodes</h1>
      </b-col>
    </b-row>

    <hr>

    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card class="mb-4 shadow" no-body v-b-toggle.collapse-1 border-variant="warning" style="border-width: 0.15rem" onfocus="this.blur();">
            <b-row no-gutter> 
              <b-col>
                <b-card-body>
                  <b-card-title>Compute Node #72</b-card-title>
                  <b-card-text>
                    <b-icon-clock-fill></b-icon-clock-fill>
                    08:12:20
                  </b-card-text>
                </b-card-body>
              </b-col>
              <b-col>
                <b-card-body style="text-align:right">
                  <b-card-subtitle>
                    <b-icon-stop-fill style="color: #ff643d"></b-icon-stop-fill>
                    Stopped
                  </b-card-subtitle>
                  <b-card-text>
                    <b-icon-cash-stack></b-icon-cash-stack>
                    £12.50
                  </b-card-text>
                </b-card-body>
              </b-col>
            </b-row>
            <b-row no-gutter class="justify-content-center">
              <b-icon-chevron-compact-down font-scale="1.5"></b-icon-chevron-compact-down>
            </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row>
      <b-collapse id="collapse-1" class="mb-4 nodeExpansion">
        <b-card class="shadow">
          <b>API Key:</b> 
          {{ user_data.api_key }}
        </b-card>
      </b-collapse>
    </b-row>

    <b-row class="justify-content-center"> 
      <b-col xs="12" lg="7">
        <b-card class="mb-4 shadow" no-body v-b-toggle.collapse-2 border-variant="completed" style="border-width: 0.15rem" onfocus="this.blur();">
          <b-row no-gutter> 
            <b-col>
              <b-card-body>
                <b-card-title>Compute Node #1001</b-card-title>
                <b-card-text>
                  <b-icon-clock-fill></b-icon-clock-fill>
                  01:22:30
                </b-card-text>
              </b-card-body>
            </b-col>
            <b-col>
              <b-card-body style="text-align:right">
                <b-card-subtitle>
                  <b-spinner small style="color: #00bf26"></b-spinner>
                  Running
                </b-card-subtitle>
                <b-card-text>
                  <b-icon-cash-stack></b-icon-cash-stack>
                  £2.25
                </b-card-text>
              </b-card-body>
            </b-col>
          </b-row>
          <b-row no-gutter class="justify-content-center">
            <b-icon-chevron-compact-down font-scale="1.5"></b-icon-chevron-compact-down>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row>
      <b-collapse id="collapse-2" class="mb-4 nodeExpansion">
        <b-card class="shadow">
          <b>API Key:</b> 
          {{ user_data.api_key }}
        </b-card>
      </b-collapse>
    </b-row>

    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card class="mb-4 shadow" no-body v-b-toggle.collapse-3 border-variant="primary" style="border-width: 0.15rem" onfocus="this.blur();">
          <b-card-body title-tag="h5">
            <b-card-title>Compute Node #4722</b-card-title>
            <b-card-sub-title class="mb-2"></b-card-sub-title>
            <b-card-text>
              <b-icon-wifi-off style="color: #fbb000"></b-icon-wifi-off>
              Not Connected
            </b-card-text>
          </b-card-body>
        </b-card>
      </b-col>
    </b-row>
    <b-row>
      <b-collapse id="collapse-3" class="mb-4 nodeExpansion">
        <b-card class="shadow">
          <b>API Key:</b> 
          {{ user_data.api_key }}
        </b-card>
      </b-collapse>
    </b-row>

    <b-row class="justify-content-center">
      <b-col xs="12" lg="6">
        <b-button block class="mb-4 shadow" v-b-toggle.collapse-new variant="primary" style="border:none" onfocus="this.blur();">
          <b-icon-plus-circle-fill></b-icon-plus-circle-fill>
          Add New Node
        </b-button>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-collapse id="collapse-new" class="mb-4 nodeExpansion">
        <b-card class="mb-4 shadow" no-body style="border:none" onfocus="this.blur();">
          <b-card-body title-tag="h5">
            <b-card-title>Connect a New Node</b-card-title>
            <p>Execute the below script on your host compute node to connect to the Sybl servers</p>
            <b-row class="justify-content-center">
              <b-col xs="12" lg="10">
                <b-card>
                  <b-row>
                    <b-col>
                      <code>{{sample_code}}</code>
                    </b-col>
                    <b-col style="text-align:right">
                      <b-button no-body variant="dark" style="background:none; border:none; margin:0; padding:0;" onfocus="this.blur();">
                        <b-icon-clipboard-plus @click="copy()"></b-icon-clipboard-plus>
                        </b-button>
                    </b-col>
                  </b-row>
                </b-card>
              </b-col>
            </b-row>
            <br>
            <b-row class="justify-content-center">
              <b-spinner style="width: 3rem; height: 3rem;" label="Large Spinner" type="grow"></b-spinner>
            </b-row>
          </b-card-body>
        </b-card>
        </b-collapse>
      </b-col>
    </b-row>
  </b-container>
</template>

<style>

.nodeExpansion{
  width:100%;
}
</style>

<script>
import axios from "axios";

export default {
  data() {
    return {
      user_data: {},
      sample_code: "wget test-code.com/code",
    }
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let data = await axios.get(
        `http://localhost:3001/api/users/${user_id}`
      );
      this.user_data = data.data
    } catch (err) {
      console.log(err);
    }
  },
  methods: {
    async copy(){
      await navigator.clipboard.writeText(this.sample_code);
    }
  },
};
</script>
