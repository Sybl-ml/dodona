<template>
  <b-container fluid>
    <b-row class="justify-content-center text-center">
      <b-col lg="2" md="6" sm="8" xs="12">
        <br /><br />
        <icon-logo height="5em" width="5em" :show_text="false" />
        <h1>Confirm Email and Password</h1>
        <b-form class="mt-5 mb-3" @submit.prevent="onSubmit">
          <b-form-input
            class="mb-3"
            type="email"
            required
            placeholder="Enter Email"
            v-model="email"
          />
          <b-form-input
            type="password"
            required
            placeholder="Password"
            class="mb-3"
            v-model="password"
          />
          <b-button variant="primary" type="submit" block> PROCEED </b-button>
        </b-form>
        <a href="/forgot">Forgotten Password?</a>
        <p v-show="!valid_credentials">Incorrect Username or Password</p>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
export default {
  data() {
    return {
      email: "",
      password: "",
      valid_credentials: true,
    };
  },
  components: {
    IconLogo,
  },
  methods: {
    async onSubmit() {
      let response = await this.$http.post(
        "api/clients/register",
        {
          id: $cookies.get("token"),
          email: this.email,
          password: this.password,
        }
      );

      if (response.status === 200) {
        this.authenticated = true;
        this.$router.push({
          name: "PrivateKey",
          params: { private_key: response.data.privKey },
        });
      } else {
        this.authenticated = false;
      }
    },
  },
};
</script>
