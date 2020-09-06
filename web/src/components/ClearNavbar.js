import React from "react"
import { Navbar, Nav,} from 'react-bootstrap';
import styled from "styled-components";
import {OutlinedPrimaryButton} from './Buttons';
import Toggle from "./Toggler"

const ClearHeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const ClearHeader = ({theme, toggleTheme}) => {
        return (
		<ClearHeaderBar sticky="top">
    		<ClearHeaderBar.Collapse className="justify-content-end">
				<Toggle theme={theme} toggleTheme={toggleTheme} />
				<ClearHeaderBar.Text>Don't have an account?</ClearHeaderBar.Text>
				<OutlinedPrimaryButton variant="primary" href="/register">CREATE ONE</OutlinedPrimaryButton>
			</ClearHeaderBar.Collapse>
  		</ClearHeaderBar>
        );
    };
export default ClearHeader;