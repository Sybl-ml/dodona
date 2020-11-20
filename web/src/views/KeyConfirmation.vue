<template>
  <b-container>
    <b-row>
      <b-col>
        <p>
          This is your private key please save this somewhere secure. This key
          will not be saved but you can request a new one anytime through the
          settings.
        </p>
      </b-col>
    </b-row>
    <b-row class="justify-content-center mb-5">
      <b-col xs="12" lg="10">
        <b-card class="shadow">
          <b-row>
            <b-col md="10">
              <code>{{ private_key }}</code>
            </b-col>
            <b-col style="text-align: right" md="2">
              <b-button
                variant="dark"
                style="background: none; border: none; margin: 0; padding: 0"
              >
                <b-icon-clipboard-plus
                  @click="copy(private_key)"
                ></b-icon-clipboard-plus>
              </b-button>
            </b-col>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col cols="1">
        <b-button variant="primary" @click="nodes"> Continue </b-button>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
export default {
  name: "KeyConfirmation",
  props: {
    private_key: String,
  },
  methods: {
    async copy(s) {
      await navigator.clipboard.writeText(s);
    },
    nodes() {
      this.$router.push({ name: "Nodes" });
    },
  },
};
</script>
