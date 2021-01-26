<template>
  <b-container fluid>
    <b-row class="justify-content-center text-center">
      <b-col lg="2" md="6" sm="8" xs="12">
        <br /><br />
        <icon-logo height="5em" width="5em" :show_text="false" />
        <h1>Sign In To Sybl</h1>
        <b-form class="mt-5 mb-3" @submit.prevent="onSubmit">
          <b-form-input
            class="mb-3"
            type="email"
            required
            placeholder="Enter Email"
            v-model="email"
          />

          <b-input-group class="mb-3">
            <b-form-input
              :type="passwordType"
              required
              placeholder="Password"
              v-model="password"
            />
            <template #append>
              <b-input-group-text>
                <b-icon
                  style="cursor: pointer;"
                  :icon="passwordIcon"
                  @click="hidePassword = !hidePassword"
                />
              </b-input-group-text>
            </template>
          </b-input-group>

          <b-form-checkbox style="display:none;" value="me" class="mb-3" v-model="remember_password"
            >Remember Me</b-form-checkbox
          >
          <b-button variant="primary" type="submit" block> SIGN IN </b-button>
        </b-form>
        <a href="/forgot">Forgotten Password?</a>
        <p v-show="!valid_credentials">Incorrect Username or Password</p>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import axios from "axios";
export default {
  data() {
    return {
      email: "",
      password: "",
      remember_password: false,
      valid_credentials: true,
      hidePassword: true,
    };
  },
  components: {
    IconLogo,
  },
  computed: {
    passwordType() {
      return this.hidePassword ? "password" : "text";
    },
    passwordIcon() {
      return this.hidePassword ? "eye-fill" : "eye-slash-fill";
    },
  },
  methods: {
    async onSubmit() {
      let response = await axios.post("http://localhost:3001/api/users/login", {
        email: this.email,
        password: this.password,
      });

      response = response.data;

      if (response.token === "null") {
        this.authenticated = false;
      } else {
        this.authenticated = true;
        $cookies.set("token", response.token, { path: "/", sameSite: true });
        this.$router.push("dashboard");
      }
    },
  },
};
</script>
